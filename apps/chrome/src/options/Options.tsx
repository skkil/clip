import { useEffect, useState } from "react";
import {
  DEFAULT_PREFERENCES,
  getPreferences,
  setPreferences,
} from "../shared/preferences";
import { getKey } from "../shared/utils/key";

export default function Options() {
  const [leaderKey, setLeaderKey] = useState(DEFAULT_PREFERENCES.keys.leader);
  const [capturing, setCapturing] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    getPreferences().then((preferences) => {
      setLeaderKey(preferences.keys.leader);
    });
  }, []);

  useEffect(() => {
    if (!capturing) {
      return;
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();

      const key = getKey(e.key);
      if (key) {
        setLeaderKey(key);
        setPreferences({
          keys: {
            leader: key,
          },
        }).then(() => {
          setSaved(true);
        });
      }

      setCapturing(false);
      setSaved(false);
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [capturing]);

  return (
    <main>
      <h1>clip settings</h1>

      <label>
        Leader key
        <button onClick={() => setCapturing(true)}>
          {capturing ? "Press any key…" : leaderKey}
        </button>
      </label>

      <p>
        Hold this key over an element to enter copy mode (replaces the default
        "Alt" key).
      </p>

      {saved && <p>Saved.</p>}
    </main>
  );
}
