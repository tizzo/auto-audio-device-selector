import { Action, ActionPanel, Detail, Icon, Keyboard } from "@raycast/api";
import { usePromise } from "@raycast/utils";

import { applyPreferences, refreshMenuBar } from "./lib/audio";

export default function Command() {
  const { data, isLoading, error, revalidate } = usePromise(
    applyPreferences,
    [],
    {
      // True up the menu-bar item as soon as preferences have been applied.
      onData: () => refreshMenuBar(),
    },
  );

  let markdown: string;
  if (error) {
    markdown = `# Apply Preferences\n\n**Failed:** ${error.message}`;
  } else if (isLoading || !data) {
    markdown = `# Apply Preferences\n\nApplying your configured priority devices…`;
  } else {
    const lines: string[] = [];
    if (data.output_changed)
      lines.push(`- 🔊 **Output** → ${data.new_output ?? "?"}`);
    if (data.input_changed)
      lines.push(`- 🎤 **Input** → ${data.new_input ?? "?"}`);

    if (lines.length === 0) {
      markdown = `# Apply Preferences\n\n✅ All devices already match your configured preferences.`;
    } else {
      markdown = `# Apply Preferences\n\n✅ Applied changes:\n\n${lines.join("\n")}`;
    }
  }

  return (
    <Detail
      isLoading={isLoading}
      markdown={markdown}
      actions={
        <ActionPanel>
          <Action
            title="Apply Again"
            icon={Icon.Stars}
            shortcut={Keyboard.Shortcut.Common.Refresh}
            onAction={() => revalidate()}
          />
        </ActionPanel>
      }
    />
  );
}
