import { getPreferenceValues } from "@raycast/api";
import { execFile } from "child_process";
import { promisify } from "util";

const execFileP = promisify(execFile);

/** Directories prepended to PATH so a bare binary name resolves under Raycast's
 * minimal environment (it does not inherit your login shell PATH). */
const EXTRA_PATH_DIRS = [
  "/usr/local/bin",
  "/opt/homebrew/bin",
  "/usr/bin",
  "/bin",
];

export type DeviceKind = "Input" | "Output" | "Input/Output";

export interface Device {
  id: string;
  name: string;
  type: DeviceKind;
  is_default: boolean;
  is_available: boolean;
  uid: string | null;
}

export interface ListDevicesResult {
  devices: Device[];
  default_output: string | null;
  default_input: string | null;
}

export interface CurrentDevice {
  name: string;
  id: string;
  type: DeviceKind;
  uid: string | null;
}

export interface ShowCurrentResult {
  output: CurrentDevice | null;
  input: CurrentDevice | null;
}

export interface SwitchResult {
  success: boolean;
  device: string;
  is_input: boolean;
  error?: string;
}

export interface ApplyPreferencesResult {
  output_changed: boolean;
  new_output: string | null;
  input_changed: boolean;
  new_input: string | null;
}

function binaryPath(): string {
  const { binaryPath } = getPreferenceValues<{ binaryPath?: string }>();
  const trimmed = binaryPath?.trim();
  return trimmed && trimmed.length > 0 ? trimmed : "audio-device-monitor";
}

async function run(args: string[]): Promise<string> {
  const env = {
    ...process.env,
    PATH: [...EXTRA_PATH_DIRS, process.env.PATH].filter(Boolean).join(":"),
  };
  try {
    const { stdout } = await execFileP(binaryPath(), args, {
      env,
      timeout: 15_000,
    });
    return stdout;
  } catch (err) {
    const e = err as NodeJS.ErrnoException & { stderr?: string };
    if (e.code === "ENOENT") {
      throw new Error(
        `Could not find the "${binaryPath()}" binary. Set an absolute path in the extension preferences.`,
      );
    }
    throw new Error(
      e.stderr?.trim() || e.message || "audio-device-monitor failed",
    );
  }
}

async function runJson<T>(args: string[]): Promise<T> {
  const stdout = await run([...args, "--json"]);
  return JSON.parse(stdout) as T;
}

export function listDevices(): Promise<ListDevicesResult> {
  return runJson<ListDevicesResult>(["list-devices"]);
}

export function showCurrent(): Promise<ShowCurrentResult> {
  return runJson<ShowCurrentResult>(["show-current"]);
}

export function switchDevice(
  name: string,
  isInput: boolean,
): Promise<SwitchResult> {
  const args = ["switch", "--device", name];
  if (isInput) args.push("--input");
  return runJson<SwitchResult>(args);
}

export function applyPreferences(): Promise<ApplyPreferencesResult> {
  return runJson<ApplyPreferencesResult>(["apply-preferences"]);
}
