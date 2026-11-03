# Installs the helper's NSIS installer for this account (/S), starts the helper until its local pipe says hello, then
# uninstalls it and checks that the program is gone. CI runs it on the signed installer before a release is published
# (helper-release.yml, verify), where -Publisher also asks for a valid, timestamped Authenticode signature of that
# publisher on the installer and on the installed program (Get-AuthenticodeSignature: WinVerifyTrust's Authenticode
# policy, as signtool verify /pa).
# Usage: pwsh installer-check.ps1 -Setup <setup.exe> [-Publisher 'Pluxia GmbH']
param(
	[Parameter(Mandatory)] [string] $Setup,
	[string] $Publisher
)
$ErrorActionPreference = 'Stop'

function Assert-Signed([string] $Path) {
	$sig = Get-AuthenticodeSignature -FilePath $Path
	if ($sig.Status -ne 'Valid') { throw "${Path}: the signature is $($sig.Status) ($($sig.StatusMessage))" }
	$subject = $sig.SignerCertificate.Subject
	if ($subject -notmatch "(^|, )O=$([regex]::Escape($Publisher))(,|$)") { throw "$Path is signed by $subject" }
	if (-not $sig.TimeStamperCertificate) { throw "${Path}: the signature carries no timestamp" }
	"Signed: $Path, by $subject"
}

function Invoke-Silent([string] $Program, [string] $What) {
	$run = Start-Process $Program -ArgumentList '/S' -Wait -PassThru
	if ($run.ExitCode -ne 0) { throw "the $What exited with $($run.ExitCode)" }
}

$Setup = (Resolve-Path $Setup).Path
if ($Publisher) { Assert-Signed $Setup }
Invoke-Silent $Setup 'installer'
# the install folder (NSIS's per-user default, %LOCALAPPDATA%\<productName>) is also the helper's data folder
$dir = Join-Path $env:LOCALAPPDATA 'QuestPump'
$exe = Join-Path $dir 'questpump-helper.exe'
if (-not (Test-Path $exe)) { throw "the installer put no helper at $exe" }
if ($Publisher) { Assert-Signed $exe }
$helper = Start-Process $exe -PassThru
$pipe = [System.IO.Pipes.NamedPipeClientStream]::new('.', 'questpump-helper', [System.IO.Pipes.PipeDirection]::In)
try {
	$pipe.Connect(60000)
	$line = [System.IO.StreamReader]::new($pipe).ReadLine()
} finally {
	$pipe.Dispose()
	Stop-Process -Id $helper.Id -ErrorAction SilentlyContinue
}
if ($line -notmatch '"type":"hello"') { throw "the helper's pipe said: $line" }
"The helper started: $line"
Invoke-Silent (Join-Path $dir 'uninstall.exe') 'uninstaller'
# NSIS's uninstaller runs again from a temporary copy: wait for it to finish
for ($i = 0; $i -lt 60 -and (Test-Path $exe); $i++) { Start-Sleep -Seconds 1 }
if (Test-Path $exe) { throw "the uninstaller left $exe" }
"Uninstalled"
