import { useState } from "react";

function App() {
  const [finalSpeed, setFinalSpeed] = useState(0.0);
  const [loading, setLoading] = useState(false);

  const speedTest = () => {
    let testRun = 0;
    let startTime: number = 0;
    let endTime: number = 0;
    const res: number[] = [];
    let connection: WebSocket | null = new WebSocket("ws://localhost:8080/ws");
    connection.onopen = () => {
      testRun++;
      startTime = performance.now();
      connection?.send("start");
    };

    connection.onmessage = () => {
      endTime = performance.now();
      res.push(endTime - startTime);
      if (testRun < 50) {
        testRun++;
        startTime = performance.now();
        setLoading(true);
        connection?.send("start");
      } else {
        connection?.close();
        connection = null;
        const testAverage = res.reduce((x, y) => x + y) / res.length / 1000;
        const mbps = 10 / testAverage; // since fileSize is 10mb
        setLoading(false);

        setFinalSpeed(mbps);
      }
    };
  };
  return (
    <div className="m-2 p-4 text-white flex flex-col items-center justify-center">
      <button
        className="bg-pink-500 shadow-lg p-2 mb-2 rounded-md hover:bg-slate-200"
        onClick={speedTest}
      >
        Click
      </button>
      {loading === true ? (
        <div className="text-5l text-pink-500">loading</div>
      ) : (
        <div className="text-5l text-pink-500">
          {finalSpeed.toFixed(2)} MB/s
        </div>
      )}
    </div>
  );
}

export default App;
