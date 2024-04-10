import React, { useState, useEffect } from "react";
import "./App.css";

import { invoke } from "@tauri-apps/api/tauri";

import { appWindow } from "@tauri-apps/api/window";

import { Select, SelectSection, SelectItem, Button } from "@nextui-org/react";

async function fetchSetting(key) {
  const value = await invoke("get_setting", { key });
  return value;
}

async function updateSetting(key, value) {
  await invoke("set_setting", { key, value });
}

appWindow.listen("tauri://close-requested", async (event) => {
  // 阻止窗口默认的关闭行为
  //event.preventDefault();

  // 隐藏窗口
  await appWindow.hide();
});

function App() {
  // 组件状态，用于跟踪选中的交易对
  const [selectedPair, setSelectedPair] = useState("");

  useEffect(() => {
    // 这里的代码会在组件首次渲染后执行
    console.log("组件已挂载");

    const fetchData = async () => {
      const response = await fetchSetting("pair");
      console.log(`fetchData ${response}`);
      setSelectedPair(response.toString());
    };
    fetchData();
    // 如果需要，可以在这里返回一个清理函数
    return () => {
      // 这里的代码会在组件卸载前执行
      console.log("组件将卸载");
    };
  }, []); // 空依赖项数组表示这个 useEffect 只在首次渲染时运行

  // 处理下拉框选项改变的事件
  const handlePairChange = (event) => {
    console.log("handlePairChange");
    setSelectedPair(event.target.value);
  };

  // 处理表单提交的事件
  const handleSubmit = async (event) => {
    event.preventDefault(); // 阻止表单默认提交行为
    console.log(`你选择的交易对是: ${selectedPair}`);
    await updateSetting("pair", selectedPair);
  };
  const pairs = [
    { label: "BTC/USDT", value: "BTC/USDT" },
    { label: "ETH/USDT", value: "ETH/USDT" },
    { label: "LTC/USDT", value: "LTC/USDT" },
    { label: "BOME/USDT", value: "BOME/USDT" },
  ];
  return (
    <div className="flex w-full max-w-xs flex-col gap-2 m-4">
      <Select
        label="Watch Pair"
        variant="bordered"
        placeholder="Select  Pair"
        selectedKeys={[selectedPair]}
        className="max-w-xs"
        onChange={handlePairChange}
      >
        {pairs.map((pair) => (
          <SelectItem key={pair.value} value={pair.value}>
            {pair.label}
          </SelectItem>
        ))}
      </Select>
      {/*  <p className="text-small text-default-500">Selected: {selectedPair}</p> */}

      <Button onClick={handleSubmit} color="primary">
        Confirm
      </Button>
    </div>
  );
}

export default App;
