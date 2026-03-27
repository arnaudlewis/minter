export function MinterLogo({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="none">
      <style>{`
        .mlogo-block {
          fill: currentColor;
          transform-box: fill-box;
          transform-origin: center;
          animation-duration: 1.35s;
          animation-timing-function: cubic-bezier(0.22, 1, 0.36, 1);
          animation-fill-mode: forwards;
        }
        .mlogo-b1 { animation-name: mlogo-split1; }
        .mlogo-b2 { animation-name: mlogo-split2; }
        .mlogo-b3 { animation-name: mlogo-split3; }
        @keyframes mlogo-split1 {
          0%, 38% { x: 8.5px; y: 8.5px; width: 7px; height: 7px; rx: 1.4px; }
          48% { x: 8.25px; y: 8.75px; width: 7.5px; height: 6.5px; rx: 1.5px; }
          78% { x: 2.6px; y: 2.6px; width: 7px; height: 7px; rx: 1px; }
          100% { x: 3px; y: 3px; width: 7px; height: 7px; rx: 1px; }
        }
        @keyframes mlogo-split2 {
          0%, 38% { x: 8.5px; y: 8.5px; width: 7px; height: 7px; rx: 1.4px; }
          48% { x: 8.25px; y: 8.75px; width: 7.5px; height: 6.5px; rx: 1.5px; }
          78% { x: 14.4px; y: 2.6px; width: 7px; height: 7px; rx: 1px; }
          100% { x: 14px; y: 3px; width: 7px; height: 7px; rx: 1px; }
        }
        @keyframes mlogo-split3 {
          0%, 38% { x: 8.5px; y: 8.5px; width: 7px; height: 7px; rx: 1.4px; }
          48% { x: 8.25px; y: 8.75px; width: 7.5px; height: 6.5px; rx: 1.5px; }
          78% { x: 8.5px; y: 14.4px; width: 7px; height: 7px; rx: 1px; }
          100% { x: 8.5px; y: 14px; width: 7px; height: 7px; rx: 1px; }
        }
      `}</style>
      <rect className="mlogo-block mlogo-b1" x="8.5" y="8.5" width="7" height="7" rx="1.4" />
      <rect className="mlogo-block mlogo-b2" x="8.5" y="8.5" width="7" height="7" rx="1.4" />
      <rect className="mlogo-block mlogo-b3" x="8.5" y="8.5" width="7" height="7" rx="1.4" />
    </svg>
  )
}
