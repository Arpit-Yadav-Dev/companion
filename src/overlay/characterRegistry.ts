// Maps a pack id to the component that draws it.
//
// Characters used to be an if/else chain repeated in both the overlay and
// the settings gallery, so each new one meant parallel edits in two files
// that were easy to get out of sync. A registry keeps it to one entry.

import type { Component } from "svelte";

import SvgCharacter from "./SvgCharacter.svelte";
import ScoutCharacter from "./ScoutCharacter.svelte";
import MochiCharacter from "./MochiCharacter.svelte";
import PipCharacter from "./PipCharacter.svelte";
import HopCharacter from "./HopCharacter.svelte";
import FlexCharacter from "./FlexCharacter.svelte";
import BaoCharacter from "./BaoCharacter.svelte";
import BoltCharacter from "./BoltCharacter.svelte";
import SunnyCharacter from "./SunnyCharacter.svelte";
import LunaCharacter from "./LunaCharacter.svelte";
import SproutCharacter from "./SproutCharacter.svelte";
import BuzzCharacter from "./BuzzCharacter.svelte";
import NimbusCharacter from "./NimbusCharacter.svelte";
import AuraCharacter from "./AuraCharacter.svelte";

/** Every hand-drawn character, keyed by its pack id. */
export const SVG_CHARACTERS: Record<string, Component<any>> = {
  sip: SvgCharacter,
  scout: ScoutCharacter,
  mochi: MochiCharacter,
  pip: PipCharacter,
  hop: HopCharacter,
  flex: FlexCharacter,
  bao: BaoCharacter,
  bolt: BoltCharacter,
  sunny: SunnyCharacter,
  luna: LunaCharacter,
  sprout: SproutCharacter,
  buzz: BuzzCharacter,
  nimbus: NimbusCharacter,
  aura: AuraCharacter,
};

/**
 * The component for a pack id, or null when the pack isn't a hand-drawn
 * character (e.g. a downloaded frames pack, which renders differently).
 */
export function svgCharacterFor(packId: string | undefined | null): Component<any> | null {
  if (!packId) return null;
  return SVG_CHARACTERS[packId] ?? null;
}

/**
 * Which characters suit each built-in reminder, best first, used to order
 * the Character gallery when a reminder is being edited. Users can still
 * pick any character for any reminder.
 *
 * Entries may name packs that aren't installed (the video packs are
 * optional downloads), so consumers must filter against what's actually
 * present rather than trusting a name resolves.
 */
export const REMINDER_CHARACTER_SUGGESTIONS: Record<string, readonly string[]> = {
  water: ["droplet", "pip", "sip"],
  stretch: ["dancer", "kettle", "flex", "stretch"],
  walk: ["dash", "ruby", "flex", "scout"],
  posture: ["coach", "nurse", "flex"],
  eye_rest: ["koala", "nimbus", "luna"],
};

/** The first suggested character for a reminder that is actually installed. */
export function suggestedCharacterFor(
  reminderId: string,
  installedPackIds: readonly string[],
): string | null {
  const installed = new Set(installedPackIds);
  return (
    REMINDER_CHARACTER_SUGGESTIONS[reminderId]?.find((id) => installed.has(id)) ?? null
  );
}
