/* eslint-disable import/no-unresolved,import/extensions,import/order */

// browse available icons at https://icones.js.org/ (or https://iconify.design/icon-sets/)
import Loader from "~icons/gg/spinner";

import Check from "~icons/heroicons/check-20-solid";
import CheckCircle from "~icons/heroicons/check-circle-20-solid";
import CheckSquare from "@/assets/images/custom-icons/check-square.svg?raw";

import AlertCircle from "~icons/heroicons/exclamation-circle-20-solid";
import AlertSquare from "@/assets/images/custom-icons/exclamation-square.svg?raw";
import AlertTriangle from "~icons/heroicons/exclamation-triangle-20-solid";

import X from "~icons/heroicons/x-mark-20-solid";
import XCircle from "~icons/heroicons/x-circle-20-solid";
import XSquare from "@/assets/images/custom-icons/x-square.svg?raw";

import Minus from "~icons/heroicons/minus-20-solid";
import MinusCircle from "~icons/heroicons/minus-circle-20-solid";

import Plus from "~icons/heroicons/plus-20-solid";
import PlusCircle from "~icons/heroicons/plus-circle-20-solid";

import Tilde from "@/assets/images/custom-icons/tilde.svg?raw";
import TildeCircle from "@/assets/images/custom-icons/tilde-circle.svg?raw";

import QuestionMarkCircle from "~icons/heroicons-solid/question-mark-circle";
import Play from "~icons/ion/play-sharp";

import Arrow from "~icons/heroicons-solid/arrow-up";
import Chevron from "~icons/heroicons-solid/chevron-up";

import Save from "~icons/heroicons-solid/save";
import Trash from "~icons/heroicons-solid/trash";

import PlayCircle from "~icons/heroicons-solid/play";
import Beaker from "~icons/heroicons-solid/beaker";
import Link from "~icons/heroicons-solid/link";
import Moon from "~icons/heroicons-solid/moon";
import Sun from "~icons/heroicons-solid/sun";
import Eye from "~icons/heroicons-solid/eye";
import EyeOff from "~icons/heroicons-solid/eye-off";
import ClipboardCopy from "~icons/heroicons-solid/clipboard-copy";
import Refresh from "~icons/heroicons-solid/refresh";
import Pencil from "~icons/heroicons-outline/pencil";
import Cube from "~icons/heroicons-outline/cube";
import Clock from "~icons/heroicons-solid/clock";
import ExclamationCircle from "~icons/heroicons-solid/exclamation-circle";
import CreditCard from "~icons/heroicons-solid/credit-card";
import Bell from "~icons/heroicons-solid/bell";
import CheckBadge from "~icons/heroicons-solid/badge-check";
import DotsHorizontal from "~icons/heroicons-solid/dots-horizontal";
import DotsVertical from "~icons/heroicons-solid/dots-vertical";
import Search from "~icons/heroicons-solid/search";
import Selector from "~icons/heroicons-solid/selector";
import Lock from "~icons/heroicons-solid/lock-closed";
import LockOpen from "~icons/heroicons-solid/lock-open";
import Diagram from "~icons/raphael/diagram";

// octicons (from github) available as no suffix, -16, -24
import GitBranch from "~icons/octicon/git-branch-24";
import GitCommit from "~icons/octicon/git-commit-24";
import GitMerge from "~icons/octicon/git-merge-24";
import Tools from "~icons/octicon/tools";
import ExternalLink from "~icons/octicon/link-external";

// restricting the type here (Record<string, FunctionalComponent>) kills our IconName type below
/* eslint sort-keys: "error" */
export const ICONS = Object.freeze({
  "alert-circle": AlertCircle,
  "alert-square": AlertSquare,
  "alert-triangle": AlertTriangle,
  beaker: Beaker,
  bell: Bell,
  check: Check,
  "check-badge": CheckBadge,
  "check-circle": CheckCircle,
  "check-square": CheckSquare,
  "clipboard-copy": ClipboardCopy,
  clock: Clock,
  component: Cube,
  "credit-card": CreditCard,
  diagram: Diagram,
  "dots-horizontal": DotsHorizontal,
  "dots-vertical": DotsVertical,
  edit: Pencil,
  "exclamation-circle": ExclamationCircle,
  "external-link": ExternalLink,
  eye: Eye,
  "git-branch": GitBranch,
  "git-commit": GitCommit,
  "git-merge": GitMerge,
  "help-circle": QuestionMarkCircle,
  hide: EyeOff,
  link: Link,
  loader: Loader,
  lock: Lock,
  "lock-open": LockOpen,
  minus: Minus,
  "minus-circle": MinusCircle,
  moon: Moon,
  play: Play,
  "play-circle": PlayCircle,
  plus: Plus,
  "plus-circle": PlusCircle,
  refresh: Refresh,
  "refresh-active": Refresh,
  save: Save,
  search: Search,
  selector: Selector,
  show: Eye,
  sun: Sun,
  tilde: Tilde,
  "tilde-circle": TildeCircle,
  tools: Tools,
  trash: Trash,
  x: X,
  "x-circle": XCircle,
  "x-square": XSquare,
});
/* eslint-disable sort-keys */

// these icons are intended to be used with a specific direction, ex: "arrow--down"
// make sure the base icon is pointing up!
export const SPINNABLE_ICONS = Object.freeze({
  arrow: Arrow,
  // triangle: Triangle,
  chevron: Chevron,
});

/*
  additional aliases which makes it easy to be more consistent with icon usage
  while still allowing us to change icons for specific cases later
*/
const ICON_NAME_ALIASES = {};

type RegularIconNames = keyof typeof ICONS;
type IconNameAliases = keyof typeof ICON_NAME_ALIASES;
type SpinnableRawIconNames = keyof typeof SPINNABLE_ICONS;
type SpinnableIconNames = `${SpinnableRawIconNames}--${
  | "left"
  | "right"
  | "up"
  | "down"}`;

export type IconNames = RegularIconNames | IconNameAliases | SpinnableIconNames;

export function getIconByName(name: string) {
  /* eslint-disable @typescript-eslint/no-explicit-any */

  const nameWithoutModifiers = name.split("--")[0];

  const icon =
    (SPINNABLE_ICONS as any)[nameWithoutModifiers] ||
    (ICONS as any)[nameWithoutModifiers] ||
    (ICONS as any)[(ICON_NAME_ALIASES as any)[nameWithoutModifiers]] ||
    ICONS["help-circle"];
  return icon as string;
}
