import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import "./App.css";
import emote from './assets/crying-yt.png';

type AudioFeature = { rms: number; zcr: number };

const DECAY = 0.78;

function App() {
  const [feature, setFeature] = useState<AudioFeature>({ rms: 0, zcr: 0 });
  const [error, setError] = useState<string | null>(null);
  const target = useRef<AudioFeature>({ rms: 0, zcr: 0 });
  const displayed = useRef<AudioFeature>({ rms: 0, zcr: 0 });

  useEffect(() => {
    const channel = new Channel<AudioFeature>();
    channel.onmessage = (msg) => {
      target.current = msg;
    };

    invoke("start_audio_stream", { onFeature: channel }).catch((e) =>
      setError(String(e)),
    );

    let raf = 0;
    const tick = () => {
      const t = target.current;
      const d = displayed.current;
      d.rms = t.rms > d.rms ? t.rms : d.rms * DECAY;
      d.zcr = t.zcr > d.zcr ? t.zcr : d.zcr * DECAY;
      setFeature({ rms: d.rms, zcr: d.zcr });
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);

    return () => {
      cancelAnimationFrame(raf);
      invoke("stop_audio_stream").catch(console.error);
    };
  }, []);

  return (
    <main className="container">
      <img
        className="emote"
        src={emote}
        alt="emote"
        style={{
          width: `${Math.min(100, feature.rms * 800)}vw`,
          height: `${Math.min(100, feature.zcr * 1500)}vh`,
        }}
      />

      <div className="hud">
        {error && <p style={{ color: "tomato" }}>{error}</p>}
        <pre>RMS: {feature.rms.toFixed(4)}</pre>
        <pre>ZCR: {feature.zcr.toFixed(4)}</pre>
      </div>
    </main>
  );
}

export default App;
