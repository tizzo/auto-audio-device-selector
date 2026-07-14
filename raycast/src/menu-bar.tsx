import { Color, Icon, MenuBarExtra, showHUD } from "@raycast/api";
import { usePromise } from "@raycast/utils";

import {
  applyPreferences,
  Device,
  listDevices,
  switchDevice,
} from "./lib/audio";

export default function Command() {
  // Visibility is controlled by Raycast's native command enable/disable
  // setting, so the component just always renders the item while enabled.
  const { data, isLoading } = usePromise(listDevices);

  const outputs = (data?.devices ?? []).filter((d) => d.type === "Output");
  const inputs = (data?.devices ?? []).filter((d) => d.type === "Input");
  const currentOutput = data?.default_output ?? undefined;
  const currentInput = data?.default_input ?? undefined;

  async function onSwitch(device: Device, isInput: boolean) {
    try {
      const result = await switchDevice(device.name, isInput);
      await showHUD(
        result.success
          ? `✓ ${isInput ? "Input" : "Output"} → ${device.name}`
          : `✗ ${result.error ?? "Switch failed"}`,
      );
    } catch (err) {
      await showHUD(`✗ ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  async function onApplyPreferences() {
    try {
      const result = await applyPreferences();
      const changes: string[] = [];
      if (result.output_changed)
        changes.push(`Output → ${result.new_output ?? "?"}`);
      if (result.input_changed)
        changes.push(`Input → ${result.new_input ?? "?"}`);
      await showHUD(
        changes.length
          ? `✓ Applied: ${changes.join(", ")}`
          : "✓ Already matching preferences",
      );
    } catch (err) {
      await showHUD(`✗ ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  const item = (
    device: Device,
    current: string | undefined,
    isInput: boolean,
  ) => (
    <MenuBarExtra.Item
      key={`${device.type}:${device.id}:${device.uid ?? device.name}`}
      title={device.name}
      icon={
        device.name === current
          ? { source: Icon.Checkmark, tintColor: Color.Green }
          : Icon.Circle
      }
      onAction={() => onSwitch(device, isInput)}
    />
  );

  return (
    <MenuBarExtra
      isLoading={isLoading}
      icon={Icon.Microphone}
      title={currentInput}
      tooltip="Audio Device Selector"
    >
      <MenuBarExtra.Section title="Output">
        {outputs.map((d) => item(d, currentOutput, false))}
      </MenuBarExtra.Section>
      <MenuBarExtra.Section title="Input">
        {inputs.map((d) => item(d, currentInput, true))}
      </MenuBarExtra.Section>
      <MenuBarExtra.Section>
        <MenuBarExtra.Item
          title="Apply Preferences"
          icon={Icon.Stars}
          onAction={onApplyPreferences}
        />
      </MenuBarExtra.Section>
    </MenuBarExtra>
  );
}
