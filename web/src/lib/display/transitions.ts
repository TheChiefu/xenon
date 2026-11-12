import { fly } from "svelte/transition";
import { cubicIn } from "svelte/easing";
import { cssTime } from "@/lib/utils";

// Slide and fade in from an offset
export function dialogFly(node: Element, params: { y?: string } = {}) {
    return fly(node, {
        y: params.y,
        duration: cssTime("--component-anim-time"),
        easing: cubicIn,
    });
}
