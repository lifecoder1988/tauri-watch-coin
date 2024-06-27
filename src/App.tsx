import { useState, useEffect, Key } from "react";
import "./App.css";

import { invoke } from "@tauri-apps/api/tauri";

import { appWindow } from "@tauri-apps/api/window";

import { Select, SelectItem } from "@nextui-org/react";

import { Tabs, Tab, Card, CardBody } from "@nextui-org/react";

import { readTextFile, BaseDirectory } from "@tauri-apps/api/fs";

async function fetchSetting(key: string) {
  const value = await invoke("get_setting", { key });
  return value;
}

async function updateSetting(key: string, value: string) {
  await invoke("set_setting", { key, value });
}

appWindow.listen("tauri://close-requested", async (event: any) => {
  // 阻止窗口默认的关闭行为
  //event.preventDefault();
  console.log(event);
  // 隐藏窗口
  await appWindow.hide();
});

interface UpdateIntervalProps {
  label: string;
  value: string;
}

const UpdateInterval: React.FC<UpdateIntervalProps> = ({ label, value }) => {
  return (
    <div className="flex flex-col justify-center p-4 max-w-full text-base leading-6 text-white bg-gray-900 w-[512px]">
      <div className="flex gap-4 justify-between max-md:flex-wrap max-md:max-w-full">
        <div>{label}</div>
        <div>{value}</div>
      </div>
    </div>
  );
};

function App() {
  // 组件状态，用于跟踪选中的交易对
  const [selectedPair, setSelectedPair] = useState("");

  const [selected, setSelected] = useState("crypto");

  const [selectedPair2, setSelectedPair2] = useState("");

  const [pairs, setPairs] = useState([
    { label: "BTC/USDT", value: "BTC/USDT" },
    { label: "ETH/USDT", value: "ETH/USDT" },
    { label: "LTC/USDT", value: "LTC/USDT" },
    { label: "BOME/USDT", value: "BOME/USDT" },
  ]);

  const [stocks, setStocks] = useState([
    { label: "上证指数", value: "sh000001" },
    { label: "沪深300", value: "sh000300" },
    { label: "隆基绿能", value: "sh601012" },
  ]);

  useEffect(() => {
    // 这里的代码会在组件首次渲染后执行
    console.log("组件已挂载");

    const fetchData = async () => {
      try {
        const conf: any = await readTextFile("app.conf.json", {
          dir: BaseDirectory.AppConfig,
        });

        const contents = JSON.parse(conf);
        console.log(BaseDirectory.AppConfig);

        setPairs(contents["pairs"]);
        setStocks(contents["stocks"]);

        console.log(stocks);
      } catch (err) {
        console.log(err);
      }

      const response = (await fetchSetting("config")) as string;
      console.log(`fetchData ${response}`);
      try {
        const config = JSON.parse(response);
        setSelected(config["type"]);
        if (config["type"] == "crypto") {
          setSelectedPair(config["value"]);
        } else {
          setSelectedPair2(config["value"]);
        }
      } catch (err) {
        console.log(err);
      }
    };
    fetchData();
    // 如果需要，可以在这里返回一个清理函数
    return () => {
      // 这里的代码会在组件卸载前执行
      console.log("组件将卸载");
    };
  }, []); // 空依赖项数组表示这个 useEffect 只在首次渲染时运行

  // 处理下拉框选项改变的事件
  const handlePairChange = (event: any) => {
    console.log(event);
    console.log(`handlePairChange ${event.target.value}`);
    if (event.target.value == "") {
      return;
    }
    setSelectedPair(event.target.value);
  };

  const handlePairChange2 = (event: any) => {
    console.log(event);
    console.log("handlePairChange2");
    if (event.target.value == "") {
      return;
    }
    setSelectedPair2(event.target.value);
  };

  // 处理表单提交的事件
  const handleSubmit = async (event: any) => {
    event.preventDefault(); // 阻止表单默认提交行为

    console.log(`你选择的交易类型是: ${selected}`);
    console.log(`你选择的交易对是: ${selectedPair}  ${selectedPair2}`);

    const data = {
      type: selected,
      value: selected == "crypto" ? selectedPair : selectedPair2,
    };
    await updateSetting("config", JSON.stringify(data));
    console.log("submit success");
  };

  const updateIntervals = [
    { label: "Price updates", value: "Every 5 seconds" },
    //{ label: "News updates", value: "Every 5 minutes" },
  ];
  return (
    <div className="flex flex-col items-center px-5 pb-14 w-full bg-gray-900 max-md:max-w-full h-screen">
      {/*  <p className="text-small text-default-500">Selected: {selectedPair}</p> */}

      <main>
        <section className="flex flex-col mt-16 font-bold text-white whitespace-nowrap max-md:mt-10">
          <h1 className="text-2xl tracking-tight">Settings</h1>
          <h2 className="mt-7 text-lg tracking-tight">General</h2>
        </section>

        <div className="flex flex-col justify-center px-4 py-3.5 max-w-full text-base leading-6 text-white bg-gray-900 w-[512px]">
          <Tabs
            aria-label="Options"
            selectedKey={selected}
            onSelectionChange={(key: Key) => setSelected(key as string)}
          >
            <Tab key="crypto" title="Web3 Crypto">
              <Card>
                <CardBody>
                  <Select
                    label="Watch Pair"
                    variant="bordered"
                    placeholder="Select Pair"
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
                </CardBody>
              </Card>
            </Tab>
            <Tab key="china" title="China">
              <Card>
                <CardBody>
                  <Select
                    label="Watch Stock"
                    variant="bordered"
                    placeholder="Select Pair"
                    selectedKeys={[selectedPair2]}
                    className="max-w-xs"
                    onChange={handlePairChange2}
                  >
                    {stocks.map((pair) => (
                      <SelectItem key={pair.value} value={pair.value}>
                        {pair.label}
                      </SelectItem>
                    ))}
                  </Select>
                </CardBody>
              </Card>
            </Tab>
          </Tabs>
        </div>

        <section className="mt-4 text-lg font-bold tracking-tight text-white">
          Update intervals
        </section>
        {updateIntervals.map((interval) => (
          <UpdateInterval
            key={interval.label}
            label={interval.label}
            value={interval.value}
          />
        ))}
        <section className="mt-4 text-lg font-bold tracking-tight text-white">
          Notifications
        </section>

        <UpdateInterval label="Price change threshold" value="2%" />
        <section className="mt-4 text-lg font-bold tracking-tight text-white">
          Privacy
        </section>

        <div className="flex flex-col mt-3 max-w-full text-sm font-bold tracking-wide leading-5 text-white w-[148px]">
          <button
            onClick={handleSubmit}
            className="flex flex-row justify-center px-4 py-2.5 mt-6 whitespace-nowrap bg-sky-700 rounded-3xl max-md:px-5"
          >
            <div className="justify-center bg-sky-700">Save</div>
          </button>
        </div>
      </main>
    </div>
  );
}

export default App;
