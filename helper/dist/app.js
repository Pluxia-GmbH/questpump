// QuestPump on this computer, the main window's page (docs/plan.md slice 35; docs/ui.md, "The desktop app"): its own
// errors into QuestPump's log first, the kit's elements defined, the views registered, the topics subscribed, then the
// keys, the live region, the kit's sounds, the frame and the first five minutes' watch over it (and the Toast that says
// the progress it found) started, and the window's open said to the counts (the app in use today).

import "./lib/errors.js"; // first: its listeners go on before any other module evaluates
import "./kit/index.js";
import "./views/index.js";
import {intent, start} from "./lib/bridge.js";
import {startFocus} from "./lib/focus.js";
import {startKeys} from "./lib/keys.js";
import {startRouter} from "./lib/router.js";
import {startSound} from "./lib/sound.js";
import {startWelcome} from "./views/welcome/welcome.js";

await start();
startFocus();
startKeys();
startSound();
startRouter(document.querySelector("qp-root-host"));
startWelcome(document.querySelector("qp-root-host"), document.querySelector("qp-toast"));
intent("usage.seen").catch(console.error);
