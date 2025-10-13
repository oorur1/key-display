import { useState, useEffect, useRef } from "react";
//import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { listen } from '@tauri-apps/api/event';
import { updateStatistics } from "./api/database";

import Stats from "./components/Stats";

interface GamepadEvent {
  type: string;
  button?: number;
  pressed?: boolean;

  axis?: number;
  direction?: string;

  count: number;
  averageReleaseTime?: number;
}

type Tab = 'mania' | 'stats' | 'setting';

{
  /*
  const ScratchComponent = () => {
    return (
      <>
        <div className="scratch-container">
          <div className={`scratch scratch-top ${isTopRotating ? 'rotating' : ''}`}></div>
          <div className={`scratch scratch-bottom ${isBottomRotating ? 'rotating' : ''}`}></div>
        </div>
      </>
    );
  }
  */
}

const KeysComponent = ({ pressed }: { pressed: Array<boolean> }) => {

  return (
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

  );
}

function App() {
  // コントローラーに関するState
  const [pressed, setPressed] = useState(Array(7).fill(false));
  const [averageReleaseTime, setAverageReleaseTime] = useState(0);
  const [rotation, setRotation] = useState(0.0);
  const [isTopRotating, setIsTopRotating] = useState(false);
  const [isBottomRotating, setIsBottomRotating] = useState(false);
  const [count, setCount] = useState(0);
  const [isPlayerOneSide, setIsPlayerOneSide] = useState(true);

  // UIに関するState 
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [activeTab, setActiveTab] = useState<Tab>('mania');

  // UseEfectを一度だけ実行する
  const once = useRef(false);

  const saveStatistics = async () => {
    const today = new Date().toISOString().split('T')[0];
    try {
      await updateStatistics(today, count);
      console.log('Statistics saved:');
    } catch (error) {
      console.error('Failed to save statistics:', error);
    }
  }

  async function setupGamepadListener() {
    const unlisten = await listen<GamepadEvent>('gamepad-input', event => {
      // ボタンの処理
      if (event.payload.type == "button" && event.payload.button !== undefined) {
        const buttonIndex = event.payload.button;
        const isPressed = event.payload.pressed;
        // 押したとき
        if (isPressed) {
          setPressed(prevPressed => {
            const newPressed = [...prevPressed];
            newPressed[buttonIndex] = true;
            return newPressed;
          });
          setCount(event.payload.count);
        }
        // リリース
        else if (event.payload.averageReleaseTime !== undefined) {
          setPressed(prevPressed => {
            const newPressed = [...prevPressed];
            newPressed[buttonIndex] = false;
            return newPressed;
          });
          setAverageReleaseTime(event.payload.averageReleaseTime);
          setCount(event.payload.count);
        }
      }
      // スクラッチの処理
      if (event.payload.type == "scratch" && event.payload.axis !== undefined && event.payload.direction !== undefined) {

        const newRotation = Math.ceil((32768.0 + event.payload.axis) / 65536.0 * 360.0);
        setRotation(newRotation);
        setCount(event.payload.count);

        const direction = event.payload.direction;
        if (direction == "left") {
          setIsTopRotating(true);
          setIsBottomRotating(false);
        } else if (direction == "right") {
          setIsTopRotating(false);
          setIsBottomRotating(true);
        } else if (direction == "neutral") {
          setIsTopRotating(false);
          setIsBottomRotating(false);
        }
      }
    })
    return unlisten;
  }

  // Gamepad listenerの起動
  useEffect(() => {
    if (once.current) return;
    once.current = true;

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


  return (
    <>
      <div className="container">
        <div className="main-content">
          {
            // mania用レイアウト
            activeTab === 'mania' && (
              <>
                <div className="mania-layout">
                  {
                    isPlayerOneSide ? (
                      <>
                        <div className="scratch-container">
                          <div className={`scratch scratch-top ${isTopRotating ? 'rotating' : ''}`}></div>
                          <div className={`scratch scratch-bottom ${isBottomRotating ? 'rotating' : ''}`}></div>
                        </div>
                        <KeysComponent pressed={pressed} />
                      </>
                    ) : (
                      <>
                        <KeysComponent pressed={pressed} />
                        <div className="scratch-container player-two">
                          <div className={`scratch scratch-top ${isTopRotating ? 'rotating' : ''}`}></div>
                          <div className={`scratch scratch-bottom ${isBottomRotating ? 'rotating' : ''}`}></div>
                        </div>
                      </>
                    )
                  }

                  <div className="change-button-container" onClick={() => { setIsPlayerOneSide(!isPlayerOneSide) }}>
                    <img src="/change_icon.png" className="change-icon">
                    </img>
                  </div>
                </div>

                <div className="information-container">
                  <p>
                    {count}
                  </p>
                  <p>
                    Release : {averageReleaseTime}
                  </p>
                </div>
              </>
            )
          }
          {
            activeTab === 'stats' && (
              <>
                <Stats />
              </>
            )
          }
          {
            activeTab === 'setting' && (
              <></>
            )
          }


        </div>
        {
          sidebarOpen && (
            <div className="sidebar">
              <div className={`sidebar-item ${activeTab === "mania" ? "active" : ""}`} onClick={() => setActiveTab('mania')}>メイン</div>
              <div className={`sidebar-item ${activeTab === "stats" ? "active" : ""}`} onClick={() => setActiveTab('stats')}>統計</div>
              <div className={`sidebar-item ${activeTab === "setting" ? "active" : ""}`} onClick={() => setActiveTab('setting')}>設定</div>
            </div >
          )
        }

      </div >
    </>
  );
}

export default App;
