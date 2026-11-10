# QuestPump

Every quest you turn in asks for a few moves at your desk. Beat your own records. A character sheet for your body.

QuestPump is an add-on made for World of Warcraft: Forever. It also loads in the current World of Warcraft.

## What happens at a turn-in

Turn in a quest and stand still. The screen dims and asks for a pump: a few push-ups, squats, stretches or yoga moves
beside your desk. Do them, type your count, and next time the goal is just past your recent best. Your first set of a
move is your first record.

A small quest asks for one or two moves, a big one a circuit of three. Set Moves per quest to Light for one move a
quest. There are more than 300 moves, and most need no gear. They sit on 22 ladders in 7 colours, from wall push-ups
up.

## What you control

- **Bank all:** one click, and the moves wait for your next pump.
- **Pause:** for an hour, four hours, until 04:00, or until you turn it back on.
- **Skip:** drop any unfinished move with one click. It earns nothing and costs nothing.
- **Your space:** tell it your floor, your room to move and your gear. Turn on quiet mode, or switch off jumping.
- **Any move off:** untick a move or a group, and pumps leave it out.
- **Something hurts?** Flag the joint, and your pumps leave out the moves that use it.

In a fight the pump waits until the fight is over. In a dungeon or a raid its moves stay unfinished and join your next
pump.

## Honour system

You type your count, and nobody checks it. It's your record.

## The two apps

QuestPump on your computer and the phone app are free and strongly recommended. Basically a must.

- **QuestPump for Mac and Windows** adds the add-on to the game and keeps both up to date. It keeps a copy of your
  progress and runs pumps on your screen when the game is closed. It never presses a key, never reads the game's memory
  and never sends input into the game. Get it at questpump.com.
- **The phone and watch app**, for iPhone and Apple Watch or Android and Wear OS, comes later. It counts down each pump
  on your phone or your wrist.

With a Bluetooth heart-rate strap on your computer, each pump gets a heart-rate check mark. Once the phone app is out,
a watch gives the check mark too.

At your first login the add-on shows a setup screen. Until the phone app is out, it asks only for QuestPump on your
computer. Install it and open it, then press Check again, and the step gets its tick.

You can play without it. The add-on alone runs every pump, with records, XP and the sheet. QuestPump on your computer
adds a copy of your progress, the check mark with a strap, and pumps on your screen when the game is closed. The phone
app will add the countdown on your phone or wrist and catch-up pumps on the phone. Type /qp setup to see the setup
screen again.

## Saving your progress on Forever

QuestPump on your computer keeps a copy of your progress in the game's add-on folder. At each login the add-on reads
that copy when it is newer than the game's own save. So your progress comes back even if the game skips loading an
add-on's save, as the Forever beta did for a time (forum thread 2354798). Without QuestPump on your computer,
/qp export gives you a string of your progress, and /qp import brings it back.

## Your gear

Most moves need no gear. Tick what you have, and pumps add the moves that use it: a stable chair, a sturdy table, a
towel and a door, a pull-up bar or a fixed pull-up bar, bands with a door anchor, a kettlebell, dumbbells, a barbell
and rack, a jump rope, an exercise bike, stairs, a walking pad, a step or box, an ab wheel, parallettes, rings or
straps, a weighted vest, a medicine ball, a foam roller, a backpack, and a pole outdoors.

## Commands

Type /qp help in the game for the full list. A few to start with:

- /qp pump: do your unfinished moves now, or start a pump
- /qp try: a practice move of 20 s, nothing stored
- /qp setup: the setup screen at any time
- /qp gear: your gear, your space, quiet mode and the Body switches
- /qp safety: the safety note

## Safety

A game, not medical advice. Stop and get help for chest pain, fainting, unusual breathlessness or a racing heart.
Severe muscle pain, swelling or dark urine after a session needs a doctor that day. Ask a clinician first if you have
heart, lung, kidney or metabolic disease or high blood pressure, are pregnant or had recent surgery.

## Windows says the installer is unknown

Windows SmartScreen may warn about a new app, even a signed one, until enough people have installed it. In Microsoft's
words: "There is no exact threshold, but it can take several weeks and hundreds of clean installs from a wide
audience." Click **More info**, check that the publisher reads Pluxia GmbH, then click **Run anyway**.

## In this repository

The add-on sits at the top of the repository, with its locales, its media and the quests' lines in `QuestPump_Lines/`.
The move figures in `QuestPump_FigArmor/` and the voice pack in `QuestPump_Voice/` (its takes are in `Media/Voice/`)
are each a download of their own beside the add-on. QuestPump for Mac and Windows has a folder of its own, in Rust and
Tauri.
`contract/` holds the files the add-on and the two apps share. The workflows in `.github/workflows/` release each part
from a tag of its own.

Interface: 16001 (World of Warcraft: Forever) and 120105 (the current World of Warcraft).

## Licence, names and security

The code is open source under the MIT licence (`LICENSE`), copyright 2026 Pluxia GmbH. The QuestPump name and logo
are not part of it (`TRADEMARKS.md`). To report a security problem, email security@questpump.com
(`.github/SECURITY.md`).

QuestPump by Pluxia GmbH, c/o RB Swiss Group GmbH, Blegistrasse 7, 6340 Baar, Switzerland. Support:
contact@questpump.com

World of Warcraft is a trademark of Blizzard Entertainment, Inc. QuestPump is not affiliated with or endorsed by
Blizzard Entertainment.
