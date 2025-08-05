import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { listen } from '@tauri-apps/api/event';

interface GamepadEvent {
  type: string;
  button?: number;
  pressed?: boolean;
  axis?: number;
}

function App() {
  const [pressed, setPressed] = useState(Array(7).fill(false));
  const [rotation, setRotation] = useState(0.0);
  const [isTopRotating, setIsTopRotating] = useState(false);
  const [isBottomRotating, setIsBottomRotating] = useState(false);

  async function setupGamepadListener() {
    const unlisten = await listen<GamepadEvent>('gamepad-input', event => {
      if (event.payload.type == "button" && event.payload.button !== undefined) {
        const buttonIndex = event.payload.button;
        const isPressed = event.payload.pressed;
        if (isPressed) {
          setPressed(prevPressed => {
            const newPressed = [...prevPressed];
            newPressed[buttonIndex] = true;
            return newPressed;
          });
        }
        else {
          setPressed(prevPressed => {
            const newPressed = [...prevPressed];
            newPressed[buttonIndex] = false;
            return newPressed;
          });
        }
      }
      if (event.payload.type == "scratch" && event.payload.axis !== undefined) {
        const newRotation = (32768.0 + event.payload.axis) / 65536.0 * 360.0;
        setRotation(prevRotation => {
          //TODO: 0->360 もしくは 360 -> 0で逆方向の回転が入る
          if (prevRotation > newRotation) {
            setIsBottomRotating(true);
            setIsTopRotating(false);
          } else if (prevRotation < newRotation) {
            setIsTopRotating(true);
            setIsBottomRotating(false);
          }

          return newRotation;
        });
      }
    })
    return unlisten;
  }

  // Gamepad listenerの起動
  useEffect(() => {
    let unlistenFn: (() => void) | null = null;

    setupGamepadListener().then(unlisten => {
      unlistenFn = unlisten;
    });

    // クリーンアップ関数
    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    }
  }, []);

  // Scratch
  useEffect(() => {
    console.log(isBottomRotating);
    if (isTopRotating) {
      const timer = setTimeout(() => {
        setIsTopRotating(false);
      }, 50);

      return () => clearTimeout(timer);
    } else if (isBottomRotating) {
      const timer = setTimeout(() => {
        setIsBottomRotating(false);
      }, 50);

      return () => clearTimeout(timer);
    }
  }, [rotation]);

  return (
    <>
      <div className="container">
        <div className="main-layout">

          <div className="scratch-container">
            <div className={`scratch scratch-top ${isTopRotating ? 'rotating' : ''}`}></div>
            <div className={`scratch scratch-bottom ${isBottomRotating ? 'rotating' : ''}`}></div>
          </div>

          <div className="keys-container">
            <div className="row">
              {
                pressed.map((isPressed, index) => {
                  const is_blue_key = (index: number) => {
                    if (index === 1 || index === 3 || index === 5) {
                      return <div key={index} className={isPressed ? "key-pressed-blue" : "key"}></div>;
                    } else {
                      return;
                    }
                  }

                  return (is_blue_key(index))
                })
              }
            </div>

            <div className="row">
              {
                pressed.map((isPressed, index) => {
                  const is_blue_key = (index: number) => {
                    if (index === 0 || index === 2 || index === 4 || index === 6) {
                      return <div key={index} className={isPressed ? "key-pressed-white" : "key"}></div>;
                    } else {
                      return;
                    }
                  }

                  return (is_blue_key(index))
                })
              }
            </div>
          </div>
        </div>
      </div >
    </>
  );
}

export default App;
