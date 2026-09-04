import { describe, expect, it } from "vitest";

import {
  B_DOWN,
  B_LEFT,
  B_RIGHT,
  B_SHOOT,
  B_UP,
  H_BACK,
  H_BOMB,
  H_COIN,
  H_CONFIRM,
  H_RIGHT,
  createInput,
} from "../../src/engine/input";

function press(key: string, init: KeyboardEventInit = {}): void {
  window.dispatchEvent(new KeyboardEvent("keydown", { key, ...init }));
}

function release(key: string): void {
  window.dispatchEvent(new KeyboardEvent("keyup", { key }));
}

describe("input", () => {
  it("reports held movement for as long as the key is down", () => {
    const input = createInput();
    press("ArrowLeft");
    expect(input.read() & B_LEFT).toBeTruthy();
    expect(input.read() & B_LEFT).toBeTruthy();
    release("ArrowLeft");
    expect(input.read() & B_LEFT).toBeFalsy();
    input.dispose();
  });

  it("reports a press exactly once", () => {
    const input = createInput();
    press("ArrowRight");
    expect(input.read() & H_RIGHT).toBeTruthy();
    expect(input.read() & H_RIGHT).toBeFalsy();
    input.dispose();
  });

  it("maps WASD onto the arrows", () => {
    const input = createInput();
    press("w");
    press("s");
    press("a");
    press("d");
    const bits = input.read();
    expect(bits & B_UP).toBeTruthy();
    expect(bits & B_DOWN).toBeTruthy();
    expect(bits & B_LEFT).toBeTruthy();
    expect(bits & B_RIGHT).toBeTruthy();
    input.dispose();
  });

  it("treats the shot key as the menu confirm, as the cabinet did", () => {
    const input = createInput();
    press(" ");
    const bits = input.read();
    expect(bits & B_SHOOT).toBeTruthy();
    expect(bits & H_CONFIRM).toBeTruthy();
    input.dispose();
  });

  it("gives Enter confirm without firing a shot", () => {
    const input = createInput();
    press("Enter");
    const bits = input.read();
    expect(bits & H_CONFIRM).toBeTruthy();
    expect(bits & B_SHOOT).toBeFalsy();
    input.dispose();
  });

  it("makes bomb, pause and coin one action per press", () => {
    const input = createInput();
    press("x");
    press("Escape");
    press("c");
    const bits = input.read();
    expect(bits & H_BOMB).toBeTruthy();
    expect(bits & H_BACK).toBeTruthy();
    expect(bits & H_COIN).toBeTruthy();
    expect(input.read()).toBe(0);
    input.dispose();
  });

  it("does not repeat a held bomb", () => {
    const input = createInput();
    press("x");
    expect(input.read() & H_BOMB).toBeTruthy();
    press("x", { repeat: true });
    expect(input.read() & H_BOMB).toBeFalsy();
    input.dispose();
  });

  it("keeps fullscreen and mute away from the game", () => {
    const input = createInput();
    press("f");
    press("m");
    expect(input.read()).toBe(0);
    const shell = input.takeShellKeys();
    expect(shell).toEqual({ full: true, mute: true });
    expect(input.takeShellKeys()).toEqual({ full: false, mute: false });
    input.dispose();
  });

  it("leaves browser chords alone", () => {
    const input = createInput();
    press("f", { metaKey: true });
    press("ArrowLeft", { ctrlKey: true });
    expect(input.read()).toBe(0);
    expect(input.takeShellKeys().full).toBe(false);
    input.dispose();
  });

  it("lets go of everything when the window loses focus", () => {
    const input = createInput();
    press("ArrowLeft");
    press(" ");
    input.read();
    window.dispatchEvent(new Event("blur"));
    expect(input.read()).toBe(0);
    input.dispose();
  });

  it("stops listening once disposed", () => {
    const input = createInput();
    input.dispose();
    press("ArrowLeft");
    expect(input.read()).toBe(0);
  });
});
