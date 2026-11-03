//! Hero on the computer (docs/ui.md, "Hero, the Book and the Log"; slices 27 and 30 on the phones, iOS `SheetTab`,
//! `HeroPages` and `FocusWindow`, Android's twins): the game's own sheet once the player has one (the newest
//! character's, as `GET /v1/sheet` serves it) and the app's own fold otherwise (the engine's `summary`, which
//! `desk/home.rs` keeps), from the same rows; the body map lit by muscle from both (the core's `HeroBody`: each muscle
//! the higher colour of the two, the move behind it with the higher record), the stats, the level and its XP with the
//! rested part, the title (on the app's own sheet chosen in the title window among the titles its fold holds earned,
//! kept in the shared body, which the engine's profile takes) and the beast, the focus (a row: `hero.focus` writes it
//! through the engine's `focus_row` and `Session::keep_computer`; at QP 10 the focus window asks once a launch, as an
//! app pump that ended done leaves the screen, as the phones' `HomeView.askFocus`), the streak's chip, and the pages a
//! muscle, a stat and the level open, their ladder lines from the engine's Book rows (`questpump_engine::BookLadder`,
//! the phones' `HeroBody.ladder`) with only a muscle's own move picked here. Pro's charts on those pages (the
//! `hero.charts` topic: the level, the stats and their weekly gains, each muscle's best rung and its moves per week,
//! from the sheets the game folded, the weeks and the record days the history's series counts, `desk/history.rs`) only
//! while `Helper::pro_on()` is `Some(true)`; without Pro the topic is null and the pages draw no chart and no teaser.
//!
//! The words the game's sheet names its parts with (contract/sheet.json: the stats, the muscles, the qualities, the
//! ladders, the focuses, the titles and the beasts) and the library's facts (contract/library.json,
//! contract/moves.json) are the built-in facts' (`desk/facts.rs`), Hero's own rules over them here (the levels, the
//! rested XP, the ladders a page lists); a file that does not read shows Hero and Pump home without its words or facts.
//! The page words the rest through copy.js.
//!
//! The `hero` topic: `{summary, own, source, name, title, title_key, titles, grammar, level, beast, focus, focuses,
//! ask_focus, streak, stats, figure, muscles, pro}` (`HeroView`: each muscle and each stat carries its page's ladder
//! lines, a muscle how to raise it); `hero.charts`: `HeroCharts` or null. Intents: `hero.focus {focus}` (a focus code
//! of contract/sheet.json's `focus`), `hero.focus.asked` (the QP 10 ask showed: not again this launch) and `hero.title
//! {key}` (a title's key of contract/sheet.json's `titles`, or null for none).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use questpump_core::body::Body;
use questpump_core::codes::{kind, mark, outcome};
use questpump_core::link::Sheet;
use questpump_core::protocol::RowRecord;
use questpump_core::row::Row;
use questpump_engine::{BookLadder, SetAside};
use serde::Serialize;
use serde_json::Value;
use tauri::Manager as _;

use super::Intent;
use super::facts::{self, Facts, Focus, facts};
use super::history::{self, GameDay};
use super::home::Board;
use super::menu_pages::setup::stamped;
use super::seams::StoreMark;
use crate::lan;
use crate::runtime::Helper;
use crate::ui::Dot;

/// Hero's topic.
pub(crate) const HERO: &str = "hero";
/// Pro's charts on Hero's pages: null without Pro.
pub(crate) const CHARTS: &str = "hero.charts";
/// What the log calls the focus row.
const FOCUS_ROW: &str = "focus"; // copy-lint: not shown (a log line's word)

// Hero's rules over the contract's facts ------------------------------------------------------------------------------

/// A move as Hero's words name it: its id, its name and its key (its tile's pose, `qp-move-tile`'s `move`).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct MoveName {
	pub(crate) id: u16,
	pub(crate) name: String,
	pub(crate) key: String,
}

/// The rules of the game's sheet only Hero reads (the levels, the rested XP, the ladders a muscle's and a stat's page
/// list), over the built-in facts (`desk/facts.rs`).
impl Facts {
	/// A move's name with its id and key; none for a move of a newer library.
	fn move_name(&self, id: u16) -> Option<MoveName> {
		let name = self.name(id)?.to_owned();
		Some(MoveName { id, name, key: self.text(id).map(|t| t.key.clone()).unwrap_or_default() })
	}

	/// The ladder whose rung the rung rule compares a move with (ns.ScaleLadder): its own, else cardio's for a cardio
	/// move, the hinge ladder for the hinge pattern, else its region's.
	fn scale_ladder(&self, m: &facts::Move) -> Option<u8> {
		if m.ladder.is_some() {
			return m.ladder;
		}
		if m.dose == "cardio" {
			return Some(self.library.cardio);
		}
		if m.pattern == "hinge" {
			return self.library.ladders.iter().find(|l| l.key == "hinge").map(|l| l.id);
		}
		m.region.as_ref().and_then(|r| self.library.region_ladder.get(r)).copied()
	}

	/// What opens a family not open yet, in words: the engine's rule (`questpump_engine::Opening`, ns.OpensAt) with
	/// its move's or its ladder's name.
	fn opening(&self, opens: &questpump_engine::Opening) -> Opening {
		match *opens {
			questpump_engine::Opening::Record { r#move, count, seconds } => {
				Opening::Record { name: self.name(r#move).unwrap_or_default().to_owned(), count, seconds }
			}
			questpump_engine::Opening::Pose { r#move, ask } => {
				Opening::Pose { name: self.name(r#move).unwrap_or_default().to_owned(), perfect: ask >= mark::PERFECT }
			}
			questpump_engine::Opening::Step { ladder, step } => {
				Opening::Step { ladder: self.ladder_name(ladder).to_owned(), step: i64::from(step) }
			}
		}
	}

	/// A ladder the screens show: not retired, its moves landed.
	fn shows(&self, id: u8) -> bool {
		self.ladder_words(id).is_some_and(|l| l.kind != "retired" && l.dose != "none")
	}

	/// The ladders whose moves train a muscle as primary, in the ladders' order by stat (`ns.LADDER_ORDER`).
	fn ladders_training(&self, muscle: u8) -> Vec<u8> {
		let ids: BTreeSet<u8> = self
			.library
			.moves
			.values()
			.filter(|m| m.mp.contains(&muscle))
			.filter_map(|m| self.scale_ladder(m))
			.collect();
		self.sheet.order.iter().copied().filter(|id| ids.contains(id) && self.shows(*id)).collect()
	}

	/// The ladders that feed a stat, in the ladders' order by stat (`ns.LADDER_ORDER`).
	fn ladders_raising(&self, stat: &str) -> Vec<u8> {
		self.sheet
			.order
			.iter()
			.copied()
			.filter(|&id| self.ladder_words(id).and_then(|l| l.stat.as_deref()) == Some(stat) && self.shows(id))
			.collect()
	}

	/// The first step of the next quality after `quality` (0 none yet); none at the top.
	fn next_step(&self, quality: u8) -> Option<i64> {
		let steps = i64::try_from(self.constants().quality_by_step.len()).unwrap_or(0);
		(1..=steps).find(|&s| u8::try_from(s).ok().and_then(|s| self.step_quality(s)).unwrap_or(0) > quality)
	}

	/// The XP a level needs (ns.XPToNext).
	fn xp_to_next(&self, level: i64) -> i64 {
		let c = self.constants();
		#[expect(clippy::cast_possible_truncation, clippy::cast_precision_loss, reason = "levels and XP stay small")]
		let xp = (c.xp_base * (level as f64).powf(c.xp_exp) + 0.5).floor() as i64;
		xp
	}

	/// The level a lifetime XP reaches (ns.LevelFor), up to the cap.
	fn level_for(&self, xp: i64) -> i64 {
		let (mut level, mut xp) = (1, xp);
		while level < self.constants().level_cap && xp >= self.xp_to_next(level) {
			xp -= self.xp_to_next(level);
			level += 1;
		}
		level
	}

	/// The rested XP of a summary at `now` (Stats.lua ns.Rested): its pool with the time away since its last move
	/// filled in, up to the cap.
	fn rested(&self, s: &Summary, now: i64) -> i64 {
		let c = self.constants();
		let pool = s.rest.unwrap_or(0);
		let Some(since) = s.rt.map(|rt| now - rt) else { return pool };
		#[expect(clippy::cast_precision_loss, reason = "seconds and XP stay far under 2^52")]
		let (since_f, need) = (since as f64, self.xp_to_next(self.level_for(s.xp)) as f64);
		if since_f < c.rested_min_h * 3600.0 {
			return pool;
		}
		#[expect(clippy::cast_possible_truncation, reason = "the pool stays small")]
		let (fill, cap) = (
			(c.rested_per_day * need * since_f / 86_400.0).floor() as i64,
			(c.rested_cap_levels * need).floor() as i64,
		);
		(pool + fill).min(cap)
	}
}

// The summary ------------------------------------------------------------------------------------------------------------

/// A whole number of a summary's JSON (an integer, or a float that holds one).
pub(crate) fn whole(v: &Value) -> Option<i64> {
	#[expect(clippy::cast_possible_truncation, reason = "checked: whole, under 2^53")]
	v.as_i64().or_else(|| v.as_f64().filter(|f| f.fract() == 0.0 && f.abs() < 9.0e15).map(|f| f as i64))
}

fn list(v: &Value, key: &str) -> Vec<i64> {
	v.get(key).and_then(Value::as_array).map_or_else(Vec::new, |a| a.iter().map(|x| whole(x).unwrap_or(0)).collect())
}

fn lists(v: &Value, key: &str) -> Vec<Vec<i64>> {
	v.get(key).and_then(Value::as_array).map_or_else(Vec::new, |a| {
		a.iter()
			.map(|e| e.as_array().map_or_else(Vec::new, |n| n.iter().map(|x| whole(x).unwrap_or(0)).collect()))
			.collect()
	})
}

/// A summary's list of `["YYYY-MM-DD", n]` pairs (`dx`, `bd`) as (game day, n), in its order; a pair that does not
/// read is left out.
fn dated(v: &Value, key: &str) -> Vec<(i64, i64)> {
	v.get(key).and_then(Value::as_array).map_or_else(Vec::new, |a| {
		a.iter()
			.filter_map(|e| {
				let pair = e.as_array()?;
				Some((history::date_index(pair.first()?.as_str()?)?, whole(pair.get(1)?)?))
			})
			.collect()
	})
}

/// The XP a game sheet's fold counted on each game day it covers (contract/companion-api.yaml's `dx` and `dxFrom`, by
/// `GameDay::index`): the last `dx_days` game days at most, from `from` to the sheet's own; a covered day absent from
/// `days` gained none, a day before `from` is unknown (no XP, no level line, no week's XP).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DayXp {
	pub(crate) from: i64,
	pub(crate) days: BTreeMap<i64, i64>,
}

impl DayXp {
	/// A summary's `dx` from its `dxFrom` (no `dx` where no covered day gained XP); none from an older addon, which
	/// carries neither.
	fn read(v: &Value) -> Option<Self> {
		let from = v.get("dxFrom").and_then(Value::as_str).and_then(history::date_index)?;
		Some(Self { from, days: dated(v, "dx").into_iter().collect() })
	}

	/// The XP game day `d` gained; none before `from` or at 0.
	pub(crate) fn on(&self, d: i64) -> Option<i64> {
		(d >= self.from).then(|| self.days.get(&d).copied()).flatten().filter(|&xp| xp > 0)
	}
}

/// A ladder's entry of a summary (`lad`): its rung, the move standing for it and that move's record (its fourth number,
/// the stars, is the Book's).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Entry {
	rung: i64,
	r#move: u16,
	record: i64,
}

/// The parts of a sheet summary (Stats.lua `ns.SheetSummary`, v 6; the core's `SheetSummary`) the computer reads: the
/// game's, as the strip brought it, or the app's own, as the engine folds it, an older one's rungs as v 6 counts them
/// (`Facts::shift_entry`, `Facts::shift_earned`). Lenient: a field it lacks reads as none.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Summary {
	pub(crate) at: i64,
	pub(crate) ws: Option<i64>,
	pub(crate) level: i64,
	pub(crate) xp: i64,
	pub(crate) into: i64,
	pub(crate) need: i64,
	paragon: i64,
	rest: Option<i64>,
	pub(crate) rt: Option<i64>,
	build: Option<String>,
	title: Option<u8>,
	pub(crate) focus: Option<u8>,
	stats: Vec<i64>,
	week: Vec<i64>,
	pub(crate) streak: i64,
	pub(crate) tokens: i64,
	ladders: Vec<Entry>,
	quality: Vec<i64>,
	muscle_week: Vec<i64>,
	body: Option<String>,
	pub(crate) locked_days: Vec<i64>,
	pub(crate) active_days: Option<i64>,
	pub(crate) raw_streak: Option<i64>,
	/// The pumps that paid XP: (the pump's offered time, its XP).
	pub(crate) pump_xp: Vec<(i64, i64)>,
	/// The weekly vault at `at` (`va`): the moves done, the records or stars and since slice 55 the week's embers (the
	/// cardio minutes until then), each up to its goal's last tier, then when that week turns (unix seconds); none from
	/// an older summary.
	vault: Option<[i64; 4]>,
	/// The vault's goals the last pump reached (`lv`: 1 moves, 2 records or stars, 3 an embers tier), as they came.
	pub(crate) last_vault: Vec<i64>,
	/// Each ladder's earned steps by its id from 1 (`ear`): its best rung, then per rung the game day it was first
	/// reached (an orange or gold one's earned; `GameDay::index`), 0 at or under the rung it started on, 1 a day the
	/// save does not know. The Saga's unlocks read them (`desk::saga`).
	pub(crate) earned: Vec<Vec<i64>>,
	/// The XP the fold counted on each game day it covers (`dx` from `dxFrom`; the save is the account's, so a day's XP
	/// is never one character's sheets less another's); none from an older addon, which names no day's XP
	/// (contract/fixtures/history.json `saga` rule 15).
	pub(crate) day_xp: Option<DayXp>,
	/// The game's two best days of all time (`bd`: game day, moves), the most first; empty before a move or from an
	/// older addon (rule 4: the Saga's old best where its rows begin later).
	pub(crate) best_days: Vec<(i64, i64)>,
	/// Today's embers as the summary counted them (`em`, slice 55) with the time it counted them at (its own `at`, kept
	/// here where [`Helper::game_shown`] sets a fold of the save's `at` to the save's time); none from an addon before
	/// slice 55.
	pub(crate) embers: Option<(i64, i64)>,
}

impl Summary {
	/// A summary from its JSON; none without its version, its time and its level.
	pub(crate) fn read(v: &Value) -> Option<Self> {
		let n = |key: &str| v.get(key).and_then(whole);
		let version = n("v")?;
		// an older summary's rungs on a ladder that lost its first rung since, as the current one counts them
		let (f, ladder) = (facts(), |i: usize| u8::try_from(i + 1).unwrap_or(0));
		let mut earned = lists(v, "ear");
		for (i, e) in earned.iter_mut().enumerate() {
			f.shift_earned(ladder(i), version, e);
		}
		let ladders = lists(v, "lad")
			.into_iter()
			.enumerate()
			.map(|(i, mut e)| {
				f.shift_entry(ladder(i), version, &mut e);
				Entry {
					rung: e.first().copied().unwrap_or(0),
					r#move: e.get(1).and_then(|&m| u16::try_from(m).ok()).unwrap_or(0),
					record: e.get(2).copied().unwrap_or(0),
				}
			})
			.collect();
		Some(Self {
			at: n("at")?,
			ws: n("ws"),
			level: n("lvl")?,
			xp: n("xp").unwrap_or(0),
			into: n("into").unwrap_or(0),
			need: n("need").unwrap_or(0),
			paragon: n("par").unwrap_or(0),
			rest: n("rest"),
			rt: n("rt"),
			build: v.get("build").and_then(Value::as_str).map(str::to_owned),
			title: n("ttl").and_then(|t| u8::try_from(t).ok()),
			focus: n("fc").and_then(|f| u8::try_from(f).ok()),
			stats: list(v, "st"),
			week: list(v, "wk"),
			streak: n("streak").unwrap_or(0),
			tokens: n("tokens").unwrap_or(0),
			ladders,
			quality: list(v, "mq"),
			muscle_week: list(v, "mw"),
			body: v.get("body").and_then(Value::as_str).map(str::to_owned),
			locked_days: list(v, "lk"),
			active_days: n("ad"),
			raw_streak: n("rs"),
			pump_xp: lists(v, "lx").into_iter().filter_map(|p| Some((*p.first()?, *p.get(1)?))).collect(),
			vault: <[i64; 4]>::try_from(list(v, "va")).ok(),
			last_vault: list(v, "lv"),
			earned,
			day_xp: DayXp::read(v),
			best_days: dated(v, "bd"),
			embers: n("em").zip(n("at")).map(|(em, at)| (at, em)),
		})
	}

	/// The weekly vault at `now` (the cores' `Vault.goals`): each goal's count up to its last tier with its next
	/// tier's need (the last once reached) and its tiers (the embers' three, `vault_embers`, slice 55; one for the rest),
	/// in Stats.lua's order, the week's omen (`omen`, slice 54: its key, the card's "Hare week") and when the week
	/// turns; once `now` reached the summary's turn every goal reads 0 and the turn is the first of the following weeks
	/// still ahead. None from a summary without one.
	fn vault_at(&self, f: &Facts, now: i64, omen: Option<&str>) -> Option<Vault> {
		const WEEK: i64 = 7 * 86_400;
		let [moves, records, embers, reset] = self.vault?;
		let c = f.constants();
		let tiers = [vec![c.vault_moves], vec![c.vault_records], c.vault_embers.clone()];
		let turned = now >= reset;
		let values = if turned { [0; 3] } else { [moves, records, embers] };
		let goals = ["moves", "records", "embers"]
			.into_iter()
			.zip(values.into_iter().zip(tiers))
			.map(|(key, (value, tiers))| {
				let top = tiers.last().copied().unwrap_or(0);
				let value = value.min(top);
				let need = tiers.iter().copied().find(|&t| t > value).unwrap_or(top);
				VaultGoal { key, value, need, tiers: if tiers.len() > 1 { tiers } else { Vec::new() } }
			})
			.collect();
		Some(Vault {
			omen: omen.map(str::to_owned),
			goals,
			reset: if turned { reset + ((now - reset) / WEEK + 1) * WEEK } else { reset },
		})
	}
}

/// A goal of the weekly vault on Hero: its key (`moves`, `records`, `embers`), its count up to its last tier, its next
/// tier's need (the last once reached), and a goal of more than one tier its tiers (the embers' pips, slice 55).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct VaultGoal {
	key: &'static str,
	value: i64,
	need: i64,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	tiers: Vec<i64>,
}

/// Hero's vault card: the week's omen (its key; none where the engine did not answer), the three goals and when the
/// week turns (unix seconds).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct Vault {
	omen: Option<String>,
	goals: Vec<VaultGoal>,
	reset: i64,
}

/// The game's sheet the computer shows (`Helper::game_shown`): a character's summary with its index and name, the
/// summary as the game wrote it (the Book reads its ladders, `GameLadders`), and where it came from.
pub(crate) struct GameSheet {
	pub(crate) char: u8,
	pub(crate) name: Option<String>,
	pub(crate) summary: Summary,
	pub(crate) sheet: Value,
	pub(crate) from: SheetFrom,
}

/// Where the game's sheet the computer shows came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SheetFrom {
	/// The strip brought it: the game folded it while it ran.
	Strip,
	/// The engine folded the game's save as the game folds it at its next load (`Home::game`).
	Save,
}

/// The engine's fold of the save (`fold`, `Home::game`) with the parts only the strip's sheet of its character
/// carries (`strip`): its index in its install and its figure (`char`, `body`, `fig`, Bridge.lua `ns.SheetPayload`),
/// and the title the game chose (`ttl`) where the fold names none. The sheet the computer shows and `GET /v1/sheet`
/// serves (`Helper::game_shown`, `Helper::serve_shown`; one live character, 2026-10-09).
fn with_strip_parts(mut fold: Value, strip: Option<&Value>) -> Value {
	if let (Some(out), Some(strip)) = (fold.as_object_mut(), strip.and_then(Value::as_object)) {
		for key in ["char", "body", "fig", "ttl"] {
			if let Some(v) = strip.get(key) {
				out.entry(key).or_insert_with(|| v.clone());
			}
		}
	}
	fold
}

/// The newest character's sheet the strip brought, as `GET /v1/sheet` serves it; none before the game painted one.
fn strip_sheet(sheet: &Sheet) -> Option<GameSheet> {
	sheet
		.characters
		.iter()
		.filter_map(|c| {
			let json = c.sheet.clone()?;
			let summary = Summary::read(&json)?;
			Some(GameSheet { char: c.index, name: c.name.clone(), summary, sheet: json, from: SheetFrom::Strip })
		})
		.max_by_key(|g| g.summary.at)
}

impl Helper {
	/// The game's sheet the app shows (Hero, Pump home's streak and Today, the Book, the Log's XP and the pump's Result
	/// banner, and every phone and watch through `GET /v1/sheet`, `serve_shown`; one owner, so a game player is never
	/// shown the app's own sheet at level 1): the newest sheet the strip brought while it is at least as new as the
	/// game's newest row or the strip reads now, and its load read every app row the helper holds; else the engine's
	/// fold of the game's save as the game folds it at its next load (`Home::game`: no Screen Recording grant, a
	/// session played out of the strip's sight whose save was taken in, or app pumps since the game's load: one live
	/// character, 2026-10-09), its time the save's newest game row, its character the one the fold is for
	/// (`save_char`) and the parts only that character's strip sheet carries (`with_strip_parts`); the strip's while
	/// the engine has not answered yet. None while the game has neither a row nor a sheet: the app's own sheet shows
	/// then.
	pub(crate) fn game_shown(&self, sheet: &Sheet) -> Option<GameSheet> {
		let strip = strip_sheet(sheet);
		let Some((newest, named)) = self.newest_game_row().filter(|&(newest, _)| self.stale(strip.as_ref(), newest))
		else {
			return strip;
		};
		let char = save_char(named, strip.as_ref());
		let save = self.home.game.as_ref().and_then(|game| {
			let c = sheet.characters.iter().find(|c| c.index == char);
			let json = with_strip_parts(game.sheet.clone(), c.and_then(|c| c.sheet.as_ref()));
			let mut summary = Summary::read(&json)?;
			summary.at = json.get("saved").and_then(whole).unwrap_or(newest);
			Some(GameSheet { char, name: c.and_then(|c| c.name.clone()), summary, sheet: json, from: SheetFrom::Save })
		});
		// never below the strip's XP: before the helper read a save of that install since it started whose client
		// reads it back (the baseline the fold starts from, `Session::game_save`; never on the beta), a history from
		// before its first take-in folds short of the game's own sheet
		save.filter(|s| strip.as_ref().is_none_or(|g| s.summary.xp >= g.summary.xp)).or(strip)
	}

	/// The save the engine folds the game's sheet from (`Session::game_save`, for `Engine::game_save` before each ask
	/// of that fold) as CBOR: the character's the fold is for (`game_shown`: the newest game row's that names one, else
	/// the newest strip sheet's), with the body every device shares as the mirror carries it to the game. None before
	/// the helper follows an install, and for one that does not encode (logged).
	pub(crate) fn game_save_cbor(&self, sheet: &Sheet) -> Option<Vec<u8>> {
		let char = save_char(self.newest_game_row().map_or(0, |(_, named)| named), strip_sheet(sheet).as_ref());
		let save = self.session.game_save(char, &self.body.body)?;
		let mut cbor = Vec::new();
		ciborium::into_writer(&save, &mut cbor)
			.inspect_err(|e| log::warn!("desk: the game's save for the engine: {e}"))
			.ok()?;
		Some(cbor)
	}

	/// The week's embers Hero's vault card shows at `now` (`Summary::vault_at`'s embers goal: up to its last tier, none
	/// past its week) of the sheet Hero shows, the game's (`game`, `game_shown`) else the app's own; 0 without a vault.
	/// The "Cardio done today" toast's week is never fewer (`Helper::pump_result`, slice 55's double-check).
	pub(crate) fn vault_embers(&self, game: Option<&GameSheet>, now: i64) -> i64 {
		game.map(|g| &g.summary)
			.or(self.home.summary.as_ref())
			.and_then(|s| s.vault_at(facts(), now, None))
			.and_then(|v| v.goals.into_iter().find(|g| g.key == "embers"))
			.map_or(0, |g| g.value)
	}

	/// Whether the game's sheet the app shows needs the engine's fold of the game's save (`game_shown`): the game holds
	/// rows that no sheet the strip brought is as new as (`sheet`, `GET /v1/sheet`'s answer).
	pub(crate) fn save_sheet_wanted(&self, sheet: &Sheet) -> bool {
		self.newest_game_row().is_some_and(|(newest, _)| self.stale(strip_sheet(sheet).as_ref(), newest))
	}

	/// Whether the strip's sheet (none before the game painted one) leaves the game's rows to the engine's fold of the
	/// save: none came, or it is older than the game's newest row (at `newest`) while the strip is not read now (while
	/// it is, the game paints its sheet again within seconds of a row, and nothing is folded meanwhile), or the load
	/// that painted it never read app rows the helper holds (`Session::sheet_lacks_app_rows`: an app pump on a phone, a
	/// watch or this computer since, which the running game never hears of).
	fn stale(&self, strip: Option<&GameSheet>, newest: i64) -> bool {
		strip.is_none_or(|g| {
			(g.summary.at < newest && self.screen != Dot::Live) || self.session.sheet_lacks_app_rows(g.char)
		})
	}

	/// `GET /v1/sheet` as the app shows it (one live character, 2026-10-09): where the game's sheet the app shows is
	/// the engine's fold of the save (`game_shown`, with the parts only the strip's sheet of its character carries),
	/// that sheet stands in its character's `sheet`, so the phones and the watches show the level, the XP and the
	/// streak the game shows at its next load: a veteran's at pairing before the strip was read, and every app pump's
	/// on a day without the game.
	pub(crate) fn serve_shown(&self, sheet: &mut Sheet) {
		let Some(shown) = self.game_shown(sheet).filter(|g| g.from == SheetFrom::Save) else { return };
		if let Some(c) = sheet.characters.iter_mut().find(|c| c.index == shown.char) {
			c.sheet = Some(shown.sheet);
		}
	}

	/// The game's newest row's time (unix seconds; the catch-ups' rows among them) and the character of the newest row
	/// that names one (an alias's as the character it stands for), 0 where none does: a match pump's Bank it and its
	/// catch-up write character 0 (`Session::bank`), whose rows the game still folds at its next load. None before the
	/// game or a catch-up wrote one.
	fn newest_game_row(&self) -> Option<(i64, u8)> {
		let records = self.session.store().records();
		let newest = records.iter().map(|r| r.row_ts).max()?;
		let named = records.iter().filter(|r| r.char > 0).max_by_key(|r| r.row_ts);
		Some((i64::from(newest), named.map_or(0, |r| self.session.people().resolve(r.char))))
	}
}

/// The character the engine's fold of the save is for: `named`, the newest game row's that names one, else the
/// character of the newest strip sheet (`strip`); 0 for none.
fn save_char(named: u8, strip: Option<&GameSheet>) -> u8 {
	if named > 0 { named } else { strip.map_or(0, |g| g.char) }
}

// What the app's own fold adds ------------------------------------------------------------------------------------------

/// What the one progression says beside its summary: each move's record in the engine's fold (the core's
/// `EngineState.record`), the moves done in any row the helper keeps (`seen`, Pump home's `Days::done`) and what of the
/// game's sheet it sets aside (`set_aside`, the engine's Book's [`SetAside`]): the ladders started over, and the moves
/// whose record was cleared or whose sets this fold corrected, where the fold's record and stars stand instead of the
/// game sheet's.
pub(crate) struct Own<'a> {
	pub(crate) records: &'a HashMap<u16, i64>,
	pub(crate) seen: &'a BTreeSet<u16>,
	pub(crate) set_aside: &'a SetAside,
}

impl Own<'_> {
	fn record(&self, id: u16) -> i64 {
		self.records.get(&id).copied().unwrap_or(0)
	}
}

/// A muscle on the body map (the core's `MuscleMark`): its quality (1 Starter … 7 Artifact) and the move behind it with
/// that move's record (a hold's in seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mark {
	quality: i64,
	r#move: u16,
	record: i64,
	seconds: bool,
}

/// A sheet's colours (`HeroBody.game`): each muscle its sheet lights, with the move of the sheet's ladders that trains it
/// as primary at the highest quality (the higher record on a tie), its record the higher of the sheet's and the app's
/// own. On the game's sheet (`game`) a ladder started over and the record of a move cleared or corrected are set aside
/// (`Own::set_aside`), so the app's own fold speaks for them.
fn marks(f: &Facts, s: &Summary, own: &Own, game: bool) -> BTreeMap<u8, Mark> {
	let mut out = BTreeMap::new();
	for (index, &quality) in s.quality.iter().enumerate() {
		let Some(muscle) = u8::try_from(index + 1).ok().filter(|_| quality > 0) else { continue };
		let mut mark = Mark { quality, r#move: 0, record: 0, seconds: false };
		let mut best = 0;
		for (i, e) in s.ladders.iter().enumerate() {
			let Some(m) = f.library.moves.get(&e.r#move).filter(|m| e.r#move > 0 && m.mp.contains(&muscle)) else {
				continue;
			};
			let ladder = u8::try_from(i + 1).unwrap_or(0);
			if game && own.set_aside.ladders.contains(&ladder) {
				continue;
			}
			let q = f.rung_quality(ladder, e.rung);
			let theirs = if game && own.set_aside.moves.contains(&e.r#move) { 0 } else { e.record };
			let record = theirs.max(own.record(e.r#move));
			if q > best || (q == best && record > mark.record) {
				best = q;
				(mark.r#move, mark.record, mark.seconds) = (e.r#move, record, m.dose == "hold");
			}
		}
		out.insert(muscle, mark);
	}
	out
}

/// Each muscle's higher colour of the game's sheet and the app's own (`HeroBody.merged`): the higher record on a tie
/// (the game's on both), the record the higher of the two where both name the same move.
fn merged(game: BTreeMap<u8, Mark>, app: &BTreeMap<u8, Mark>) -> BTreeMap<u8, Mark> {
	let mut out = game;
	for (&id, &a) in app {
		let mark = match out.get(&id) {
			Some(&g) => {
				let app_wins = a.quality > g.quality || (a.quality == g.quality && a.record > g.record);
				let (mut mark, other) = if app_wins { (a, g) } else { (g, a) };
				if other.r#move == mark.r#move {
					mark.record = mark.record.max(other.record);
				}
				mark
			}
			None => a,
		};
		out.insert(id, mark);
	}
	out
}

/// What opens a family not open yet, as Book and Hero's pages say it.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Opening {
	/// "Opens at a Push-up record of 6" (a hold's count in seconds).
	Record { name: String, count: i64, seconds: bool },
	/// A pose's goal pose answered close enough, or Perfect (slice 42: the planche waits for the wrist lean, Perfect).
	Pose { name: String, perfect: bool },
	/// "Opens at Cardio step 5".
	Step { ladder: String, step: i64 },
}

/// A ladder line of a muscle's or a stat's page (the core's `LadderMark` as `LadderLines` draws it).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Line {
	ladder: u8,
	ladder_name: String,
	/// The move standing for the line; none where the library names none.
	r#move: Option<MoveName>,
	quality: u8,
	word: Option<String>,
	record: i64,
	seconds: bool,
	opens: Option<Opening>,
}

/// A ladder as Hero's pages list it (`HeroBody.ladder(_:training:among:own:)`): the engine's Book row (`mark`: the
/// entry on the higher rung of the game's sheet and the app's own, each record lifted by the app's own, the move the
/// setup stands for the rung where the entry names none, a family's opening rule while it is unplaced), and on a
/// muscle's page the ladder's move that trains `muscle` as primary (`pick`), its quality its own rung's once done, the
/// rung's for the move standing for it, none for another, its record the mark's for the move standing for the rung and
/// the app's own for another.
fn ladder_line(f: &Facts, mark: &BookLadder, muscle: Option<u8>, own: &Own) -> Line {
	let (id, standing) = (mark.ladder, mark.r#move);
	let chosen = match muscle {
		Some(muscle) => pick(f, id, muscle, own, standing, i64::from(mark.rung.max(1))),
		None => Some(standing),
	};
	let (mut shown, mut quality, mut record) = (standing, mark.quality, mark.record);
	if let Some(chosen) = chosen
		&& let Some(m) = f.library.moves.get(&chosen)
	{
		quality = if own.seen.contains(&chosen) {
			f.rung_quality(id, i64::from(m.rung))
		} else if chosen == standing {
			mark.quality
		} else {
			0
		};
		record = if chosen == standing { mark.record } else { own.record(chosen) };
		shown = chosen;
	}
	Line {
		ladder: id,
		ladder_name: f.ladder_name(id).to_owned(),
		r#move: f.move_name(shown).filter(|_| shown > 0),
		quality,
		word: f.quality_word(quality).map(str::to_owned),
		record,
		seconds: f.hold(shown),
		opens: mark.opens.as_ref().map(|o| f.opening(o)),
	}
}

/// The move of ladder `id` a muscle's page lists among those that train `muscle` as primary (`HeroBody.pick`): the
/// highest done (`own.seen`), else the one `standing` for the rung, else the highest at or under the `rung`, else the
/// lowest; the move on the higher rung, the lower id on a tie.
fn pick(f: &Facts, id: u8, muscle: u8, own: &Own, standing: u16, rung: i64) -> Option<u16> {
	let trains: Vec<(u16, &facts::Move)> = f
		.library
		.moves
		.iter()
		.filter(|(_, m)| m.mp.contains(&muscle) && f.scale_ladder(m) == Some(id))
		.map(|(&k, m)| (k, m))
		.collect();
	let highest = |list: &mut dyn Iterator<Item = &(u16, &facts::Move)>| {
		list.max_by(|a, b| (a.1.rung, b.0).cmp(&(b.1.rung, a.0))).map(|(k, _)| *k)
	};
	highest(&mut trains.iter().filter(|(k, _)| own.seen.contains(k)))
		.or_else(|| trains.iter().any(|(k, _)| *k == standing).then_some(standing))
		.or_else(|| highest(&mut trains.iter().filter(|(_, m)| i64::from(m.rung) <= rung)))
		.or_else(|| trains.iter().min_by_key(|(k, m)| (m.rung, *k)).map(|(k, _)| *k))
}

/// A ladder's line for each of `ids` the engine's Book rows hold (none before it answered).
fn lines(f: &Facts, ids: Vec<u8>, ladders: &[BookLadder], muscle: Option<u8>, own: &Own) -> Vec<Line> {
	ids.into_iter()
		.filter_map(|id| ladders.iter().find(|l| l.ladder == id))
		.map(|mark| ladder_line(f, mark, muscle, own))
		.collect()
}

// The topic ------------------------------------------------------------------------------------------------------------

/// Whose sheet Hero shows: the app's own on this computer, or the game's, folded at `at` (unix seconds).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Source {
	Computer,
	Game { at: i64 },
}

/// The level line: the level, the XP into it over what it needs, the Paragon rank past the cap, the rested XP now.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct XpLine {
	level: i64,
	into: i64,
	need: i64,
	paragon: i64,
	rested: i64,
}

/// The name line's grammar (contract/sheet.json `words.titled` and `titled_suffix`), which the page cuts at its seam.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct Grammar {
	titled: Option<String>,
	suffix: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct StatCell {
	key: String,
	name: String,
	value: i64,
	/// The gain this game week.
	week: i64,
	/// Its page's ladders, those that raise it.
	ladders: Vec<Line>,
}

/// A muscle on the map and its page.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct Muscle {
	id: u8,
	/// The sheet's name for it; none where the sheet names none.
	name: Option<String>,
	/// 0 untrained, 1 Starter … 7 Artifact.
	quality: i64,
	word: Option<String>,
	/// The move behind its colour and that move's record.
	r#move: Option<MoveName>,
	record: i64,
	seconds: bool,
	/// Its moves this week (the tooltip's line where no move explains its colour).
	week: Option<i64>,
	/// Its page: the ladders that train it, and how to raise it.
	ladders: Vec<Line>,
	raise: Option<Raise>,
}

/// How a muscle's quality rises, naming an open ladder of its page's.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Raise {
	/// "Reach step {step} on {ladder} for {quality}."
	Step { step: i64, ladder: String, quality: String },
	/// "{quality} is the top quality."
	Top { quality: String },
}

/// The streak's chip while a streak runs.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct StreakChip {
	days: i64,
	tokens: i64,
}

/// A title the title window offers: its key (the profile's `title`) and its name.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct TitleChoice {
	key: String,
	name: String,
}

/// The `hero` topic.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct HeroView {
	/// A summary stands (else the page is Hero before the first pump: QP 1, one line over the dark map).
	summary: bool,
	/// The sheet is the app's own on this computer (no game sheet yet).
	own: bool,
	source: Source,
	/// The character's name on the game's sheet; none on the app's own (the page says "Your hero").
	name: Option<String>,
	title: Option<String>,
	/// The shown title's key, the title window's check; none for no title.
	title_key: Option<String>,
	/// The titles the app's own fold holds earned, in the deeds' order (`FreeMode.titles`): on the app's own sheet the
	/// name line opens the title window with them; none on the game's, whose title is the game's choice.
	titles: Vec<TitleChoice>,
	grammar: Grammar,
	level: XpLine,
	beast: Option<String>,
	/// The focus in force by its key, Balanced before any.
	focus: String,
	focuses: Vec<Focus>,
	/// The QP 10 ask is due (`HomeView.askFocus`), at this level (library.json's `focus_level`): the focus window asks
	/// once a launch with Keep Balanced, as an app pump that ended done leaves the screen (`Helper::pump_focus_due`; never
	/// on a bare open, over a pump or under the first five minutes), while Hero shows the app's own sheet at the level
	/// or above, no focus row is folded (its summary names none) and no felt-unwell lock holds.
	ask_focus: Option<i64>,
	streak: Option<StreakChip>,
	stats: Vec<StatCell>,
	/// The body map's figure: female or male.
	figure: &'static str,
	muscles: Vec<Muscle>,
	/// The weekly vault's card under the stats, from the summary shown; none before one or from an older one.
	vault: Option<Vault>,
	/// The level's page's line under its XP bar names the rested XP too; Pro's charts only with this.
	pro: bool,
}

/// What Hero's view is made of: the game's sheet (none before the game painted one), the app's own fold's summary and
/// what its state says, its Book rows and the titles it holds earned, the streak's board, Pro, and the QP 10 ask.
struct HeroFrom<'a> {
	game: Option<&'a GameSheet>,
	own_summary: Option<&'a Summary>,
	own: Own<'a>,
	ladders: &'a [BookLadder],
	titles: &'a [u8],
	board: &'a Board,
	/// The week's omen (`Engine::embers`, slice 54): the vault card's title.
	omen: Option<&'a str>,
	pro: bool,
	ask_focus: Option<i64>,
}

/// A dated value of a chart: when (unix seconds) and the value.
type Point = (i64, f64);

/// What Pro's charts were last built from: the sheets logged, the stores' rows since the reset (the other computers'
/// among them: `Helper::store_mark`), the last sheet's time, and the game day (a new one adds its week).
pub(crate) type ChartsKey = (usize, StoreMark, Option<String>, i64);

/// Pro's charts on Hero's pages (`HistorySeries`' level, stats, gains, quality and muscle weeks; `ChartRecords`' days
/// and weeks), from the sheets the game folded, every character's on one line by the time it was folded: a value's
/// points only where it changed (the day's last sheet), a weekly value on every game week the rows count; and the
/// record days and weeks the charts star.
#[derive(Serialize, Debug, Clone, PartialEq)]
struct HeroCharts {
	level: Vec<Point>,
	stats: Vec<Vec<Point>>,
	/// Each stat's gain per game week, on every week of `weeks` (0 in a week without a sheet).
	gains: Vec<Vec<Point>>,
	quality: BTreeMap<u8, Vec<Point>>,
	/// Each muscle's moves per game week, on every week of `weeks`.
	muscle_weeks: BTreeMap<u8, Vec<Point>>,
	/// Every game week's start the rows count, ascending (`HistorySeries.weeks`): a week runs to the next one's.
	weeks: Vec<i64>,
	/// The turn of each game day with a beaten record no correction names (`ChartRecords.days`).
	record_days: Vec<i64>,
	/// The start of each game week holding such a day (`ChartRecords.weeks`).
	record_weeks: Vec<i64>,
	/// contract/sheet.json's `map_steps`: the moves a week's square counts from for each step.
	map_steps: Vec<i64>,
	/// The qualities' words (1 Starter … 7 Artifact), a muscle's best rung's line in words.
	qualities: Vec<String>,
}

/// The day's last value of each run of one key, in order.
fn last_of<'a, K: PartialEq>(sheets: &[&'a Summary], key: impl Fn(&Summary) -> K) -> Vec<&'a Summary> {
	sheets
		.iter()
		.enumerate()
		.filter(|(i, s)| sheets.get(i + 1).is_none_or(|next| key(next) != key(s)))
		.map(|(_, s)| *s)
		.collect()
}

/// A value's points over the sheets, only where it changed (the first always); a sheet without it is skipped.
fn changes(sheets: &[&Summary], value: impl Fn(&Summary) -> Option<f64>) -> Vec<Point> {
	let mut points: Vec<Point> = Vec::new();
	for s in sheets {
		let Some(v) = value(s) else { continue };
		if points.last().is_some_and(|p| (p.1 - v).abs() < f64::EPSILON) {
			continue;
		}
		points.push((s.at, v));
	}
	points
}

#[expect(clippy::cast_precision_loss, reason = "a sheet's numbers stay far under 2^52")]
fn float(v: i64) -> f64 {
	v as f64
}

fn item(list: &[i64], i: usize) -> Option<f64> {
	list.get(i).copied().map(float)
}

/// Hero's view at `now`.
fn hero_view(from: &HeroFrom, now: i64) -> HeroView {
	let f = facts();
	let (game, own_summary, own) = (from.game, from.own_summary, &from.own);
	let shown = game.map(|g| &g.summary).or(own_summary);
	let game_marks = game.map(|g| marks(f, &g.summary, own, true)).unwrap_or_default();
	let app_marks = own_summary.map(|s| marks(f, s, own, false)).unwrap_or_default();
	let colours = merged(game_marks, &app_marks);
	let muscles = f
		.sheet
		.muscles
		.iter()
		.map(|m| {
			let mark = colours.get(&m.id).copied();
			let quality = mark.map_or(0, |k| k.quality);
			let ladders = lines(f, f.ladders_training(m.id), from.ladders, Some(m.id), own);
			Muscle {
				id: m.id,
				name: f.muscle_name(m.id).map(str::to_owned),
				quality,
				word: u8::try_from(quality).ok().and_then(|q| f.quality_word(q)).map(str::to_owned),
				r#move: mark.and_then(|k| f.move_name(k.r#move).filter(|_| k.r#move > 0)),
				record: mark.map_or(0, |k| k.record),
				seconds: mark.is_some_and(|k| k.seconds),
				week: shown.and_then(|s| s.muscle_week.get(usize::from(m.id) - 1).copied()),
				raise: raise(f, u8::try_from(quality).unwrap_or(0), &ladders),
				ladders,
			}
		})
		.collect();
	let stats = f
		.sheet
		.stats
		.iter()
		.enumerate()
		.map(|(i, stat)| StatCell {
			key: stat.key.clone(),
			name: stat.name.clone(),
			value: shown.and_then(|s| s.stats.get(i).copied()).unwrap_or(0),
			week: shown.and_then(|s| s.week.get(i).copied()).unwrap_or(0),
			ladders: lines(f, f.ladders_raising(&stat.key), from.ladders, None, own),
		})
		.collect();
	// the focus in force: the app's fold's (its latest focus row, the game sheet's as its oldest), else the game
	// sheet's, Balanced before any (`FreeMode.focusKey`)
	let focus_code = own_summary.and_then(|s| s.focus).or_else(|| game.and_then(|g| g.summary.focus));
	let focus = focus_code
		.and_then(|c| f.focus(c))
		.or_else(|| f.sheet.focus.first())
		.map(|w| w.key.clone())
		.unwrap_or_default();
	let level = shown.map_or_else(
		|| XpLine { level: 1, into: 0, need: f.xp_to_next(1), paragon: 0, rested: 0 },
		|s| XpLine { level: s.level, into: s.into, need: s.need, paragon: s.paragon, rested: f.rested(s, now) },
	);
	let title = shown.and_then(|s| s.title).and_then(|t| f.title(t));
	// a title these words lack (a newer addon's) is never offered without its name
	let titles = if game.is_some() {
		Vec::new()
	} else {
		from.titles
			.iter()
			.filter_map(|&code| f.title(code))
			.map(|t| TitleChoice { key: t.key.clone(), name: t.name.clone() })
			.collect()
	};
	HeroView {
		summary: shown.is_some(),
		own: game.is_none(),
		source: game.map_or(Source::Computer, |g| Source::Game { at: g.summary.at }),
		name: game.and_then(|g| g.name.clone()),
		title: title.map(|t| t.name.clone()),
		title_key: title.map(|t| t.key.clone()),
		titles,
		grammar: Grammar { titled: f.sheet.words.titled.clone(), suffix: f.sheet.words.titled_suffix.clone() },
		level,
		beast: shown.and_then(|s| s.build.as_ref()).and_then(|b| f.sheet.builds.get(b)).cloned(),
		focus,
		focuses: f.sheet.focus.clone(),
		ask_focus: from.ask_focus,
		streak: (from.board.days > 0).then_some(StreakChip { days: from.board.days, tokens: from.board.tokens }),
		stats,
		figure: if shown.and_then(|s| s.body.as_deref()) == Some("female") { "female" } else { "male" },
		muscles,
		vault: shown.and_then(|s| s.vault_at(f, now, from.omen)),
		pro: from.pro,
	}
}

/// How to raise a muscle (`MusclePage.raise`): the next quality's step on the open ladder that stands highest, or that
/// it is at the top; nothing for an untrained one (2026-10-22: a line naming a ladder's move to start it until then)
/// or while every ladder listed is locked.
fn raise(f: &Facts, quality: u8, ladders: &[Line]) -> Option<Raise> {
	if quality == 0 {
		return None;
	}
	let open: Vec<&Line> = ladders.iter().filter(|l| l.opens.is_none()).collect();
	let (Some(step), Some(next)) = (f.next_step(quality), f.quality_word(quality + 1)) else {
		return Some(Raise::Top { quality: f.quality_word(quality).unwrap_or_default().to_owned() });
	};
	// the first of the highest on a tie, in the list's order
	let best = open.iter().fold(None::<&&Line>, |best, l| match best {
		Some(b) if b.quality >= l.quality => Some(b),
		_ => Some(l),
	});
	best.map(|l| Raise::Step { step, ladder: l.ladder_name.clone(), quality: next.to_owned() })
}

/// What the rows the helper keeps count for the charts (`HistorySeries.build` through `desk/history.rs`): every game
/// week from the first row's or app pump's to now's (from the region's weekly reset `ws`, else Monday's turn), each
/// game day with a beaten record no correction names at its turn, and each week holding one.
struct Counted {
	weeks: Vec<i64>,
	record_days: Vec<i64>,
	record_weeks: Vec<i64>,
}

fn counted(game: &[RowRecord], app: &[RowRecord], ws: Option<i64>, day: &GameDay, now: i64) -> Counted {
	let game_rows: Vec<history::Row> = game.iter().map(history::Row::of).collect();
	let app_rows: Vec<history::Row> = app.iter().map(history::Row::of).collect();
	let fixes = history::Fixes::of(&[&game_rows, &app_rows]);
	let free: Vec<(i64, i64)> = history::app_pumps(&app_rows, &fixes)
		.iter()
		.map(|p| (p.id, i64::try_from(p.moves_done()).unwrap_or(i64::MAX)))
		.collect();
	let map_steps = &facts().constants().map_steps;
	let (game_read, app_read) = (history::read(&game_rows, &fixes), history::read(&app_rows, &fixes));
	let (days, weeks) = history::series(&game_read, &free, &app_read, ws, day, map_steps, now);
	// the days the series counts a record on, each at its turn, found by a beaten row of the day
	let named: HashSet<&str> = days.iter().filter(|d| d.records > 0).map(|d| d.day.as_str()).collect();
	let record_days: BTreeSet<i64> = game
		.iter()
		.chain(app)
		.filter(|r| kind::is_move(r.kind) && r.outcome == outcome::BEAT)
		.map(|r| i64::from(r.row_ts))
		.filter(|&t| named.contains(day.name(t).as_str()))
		.map(|t| day.start(t))
		.collect();
	let weeks: Vec<i64> = weeks.iter().map(|w| w.start).collect();
	let record_weeks: BTreeSet<i64> =
		record_days.iter().filter_map(|&d| weeks.iter().rev().find(|&&w| w <= d).copied()).collect();
	Counted { weeks, record_days: record_days.into_iter().collect(), record_weeks: record_weeks.into_iter().collect() }
}

/// A weekly series on every game week of `weeks`, 0 in a week without a point (`Week.every`); as it is without weeks
/// (sheets without rows).
fn every(points: Vec<Point>, weeks: &[i64]) -> Vec<Point> {
	if weeks.is_empty() {
		return points;
	}
	let by_week: HashMap<i64, f64> = points.into_iter().collect();
	weeks.iter().map(|&w| (w, by_week.get(&w).copied().unwrap_or(0.0))).collect()
}

/// The charts from the sheets the game folded (sorted by their time), the region's weekly reset the latest of them
/// names (`reset`) and what the rows count.
fn charts(sheets: &[&Summary], reset: Option<i64>, counted: Counted, day: &GameDay) -> HeroCharts {
	let f = facts();
	let daily = last_of(sheets, |s| day.index(s.at));
	let width = |list: fn(&Summary) -> &Vec<i64>| sheets.iter().map(|s| list(s).len()).max().unwrap_or(0);
	let level =
		changes(&daily, |s| Some(float(s.level) + if s.need > 0 { float(s.into) / float(s.need) } else { 0.0 }));
	let stats = (0..width(|s| &s.stats)).map(|i| changes(&daily, |s| item(&s.stats, i))).collect();
	let quality = (0..width(|s| &s.quality))
		.filter_map(|i| Some((u8::try_from(i + 1).ok()?, changes(&daily, |s| item(&s.quality, i)))))
		.collect();
	// the week's last sheet, every week's point kept, at the start of its game week: where a sheet names the region's
	// weekly reset, the reset's own weeks (a sheet's weekly gains turn at the reset itself) at the start of the game
	// week of the same number, else the game week from Monday's turn (`HistorySeries.build`'s `bars`)
	let week = |at: i64| match reset {
		Some(r) => history::week_start(r, Some(r), day, -(at - r).div_euclid(history::WEEK_S)),
		None => history::week_start(at, None, day, 0),
	};
	let weekly = last_of(sheets, |s| week(s.at));
	let bars = |value: &dyn Fn(&Summary) -> Option<i64>| -> Vec<Point> {
		let points = weekly.iter().filter_map(|s| value(s).map(|v| (week(s.at), float(v)))).collect();
		every(points, &counted.weeks)
	};
	let gains = (0..width(|s| &s.week)).map(|i| bars(&|s: &Summary| s.week.get(i).copied())).collect();
	let muscle_weeks = (0..width(|s| &s.muscle_week))
		.filter_map(|i| Some((u8::try_from(i + 1).ok()?, bars(&|s: &Summary| s.muscle_week.get(i).copied()))))
		.collect();
	HeroCharts {
		level,
		stats,
		gains,
		quality,
		muscle_weeks,
		weeks: counted.weeks,
		record_days: counted.record_days,
		record_weeks: counted.record_weeks,
		map_steps: f.constants().map_steps.clone(),
		qualities: f.sheet.qualities.clone(),
	}
}

impl Helper {
	/// The `hero` topic now, and `hero.charts` while Pro holds.
	pub(crate) fn publish_hero(&mut self, sheet: &Sheet, board: &Board, now: i64) {
		let f = facts();
		let game = self.game_shown(sheet);
		let pro = self.pro_on() == Some(true);
		// the QP 10 ask only as an app pump that ended done leaves the screen (its cue is taken once), never on a bare
		// open, over a pump or under the first five minutes
		let due = self.pump_focus_due() && self.settings.desk.onboarded;
		let own_summary = self.home.summary.as_ref();
		let ask_focus = (due
			&& game.is_none()
			&& !self.home.focus_asked
			&& sheet.locked_until.is_none()
			&& own_summary.is_some_and(|s| s.level >= f.constants().focus_level && s.focus.is_none()))
		.then_some(f.constants().focus_level);
		let from = HeroFrom {
			game: game.as_ref(),
			own_summary,
			own: Own { records: &self.home.records, seen: &self.home.days.done, set_aside: &self.home.set_aside },
			ladders: &self.home.ladders,
			titles: &self.home.titles,
			board,
			omen: self.home.embers.as_ref().and_then(|e| e.omen.as_deref()),
			pro,
			ask_focus,
		};
		let view = hero_view(&from, now);
		self.app.state::<super::Desk>().publisher.publish(HERO, &view);
		self.publish_charts(pro, now);
	}

	/// Pro's charts, built again only when the sheets the game folded, the rows or the game day changed; null without
	/// Pro.
	fn publish_charts(&mut self, pro: bool, now: i64) {
		if !pro {
			self.home.charts_key = None;
			self.app.state::<super::Desk>().publisher.publish(CHARTS, &Value::Null);
			return;
		}
		let (log, game) = (self.session.sheet_log().records(), self.session.store().records());
		let day = &self.home.days.day;
		let key = (log.len(), self.store_mark(), log.last().map(|r| r.ts_helper.clone()), day.index(now));
		if self.home.charts_key.as_ref() == Some(&key) {
			return;
		}
		// the app rows with the other computers' beside them, rows of another source (slice 60)
		let mut app = self.session.app_rows().records().to_vec();
		app.extend(self.session.far_records(0));
		let mut read: Vec<Summary> = log
			.iter()
			.filter_map(|r| serde_json::from_str::<Value>(r.sheet.get()).ok())
			.filter_map(|v| Summary::read(&v))
			.collect();
		read.sort_by_key(|s| s.at);
		let sheets: Vec<&Summary> = read.iter().collect();
		// the region's weekly reset the latest sheet names (`HistorySeries.reset`)
		let reset = sheets.iter().rev().find_map(|s| s.ws);
		let charts = charts(&sheets, reset, counted(game, &app, reset, day, now), day);
		self.app.state::<super::Desk>().publisher.publish(CHARTS, &charts);
		self.home.charts_key = Some(key);
	}

	/// The page's intents on Hero: the focus window's choice, the QP 10 ask shown, and the title window's choice.
	pub(crate) fn hero_intent(&mut self, intent: &Intent) -> Result<(), String> {
		match intent.kind() {
			"hero.focus" => {
				let focus = intent
					.arg("focus")
					.and_then(Value::as_u64)
					.and_then(|f| u8::try_from(f).ok())
					.filter(|&f| facts().focus(f).is_some())
					.ok_or("hero.focus takes a focus of contract/sheet.json")?;
				self.engine_ready_or(intent)?;
				let at = lan::now_ms().div_euclid(1_000);
				self.engine_call(
					move |e| e.focus_row(focus, at),
					|h, answer| h.keep_own(FOCUS_ROW, answer.map(|r| vec![r])),
				);
			}
			"hero.focus.asked" => {
				self.home.focus_asked = true;
				self.home_publish();
			}
			"hero.title" => {
				let title = match intent.arg("key") {
					None | Some(Value::Null) => None,
					Some(key) => Some(
						key.as_str()
							.filter(|k| facts().sheet.titles.iter().any(|t| t.key == *k))
							.ok_or("hero.title takes a title's key of contract/sheet.json, or null")?
							.to_owned(),
					),
				};
				self.wear_title(title);
			}
			_ => return Err(intent.unknown()),
		}
		Ok(())
	}

	/// Wears `title` (a title's key of `contract/sheet.json`, none for No title): a change of the body every device
	/// shares (2026-10-09; empty for none), and the title this computer kept alone goes.
	pub(crate) fn wear_title(&mut self, title: Option<String>) {
		if self.title() != title {
			let field = self.body.body.title.as_ref();
			let change = Body { title: Some(stamped(title.unwrap_or_default(), field)), ..Body::default() };
			if self.settings.title.take().is_some() {
				self.save_settings();
			}
			self.body_changed(&change);
		}
	}

	/// Rows the engine wrote for this computer outside a pump (a focus, a check-in's answer, a pain flag), kept as a
	/// pump of their own (`d-<uuid>`, so the engine's door never merges them back) and their effects applied.
	pub(crate) fn keep_own(&mut self, what: &str, rows: Result<Vec<String>, questpump_engine::EngineError>) {
		let rows = match rows {
			Ok(rows) => rows,
			Err(e) => {
				log::warn!("desk: the {what} row: {e}");
				return;
			}
		};
		let parsed: Vec<Row> = rows.iter().filter_map(|r| r.parse().ok()).collect();
		if parsed.len() != rows.len() {
			log::warn!("desk: a {what} row the engine wrote did not read");
		}
		let id = super::pump::new_id();
		let (kept, fx) = self.session.keep_computer(&id, None, &parsed, &crate::runtime::now());
		log::info!("desk: {what}: {kept} rows kept");
		self.apply(fx);
	}
}
