#!python3

import subprocess
import sys
import json

func_name = "long-chain-rust"
gateway = "http://192.168.1.183:32227"
data_size = int(sys.argv[1]) # MB unit
tot_n = int(sys.argv[2])

def invoke_func(func_name: str, data: dict,) -> subprocess.Popen:
    p = subprocess.Popen(["faas-cli", "-g", gateway, "invoke", func_name], 
                         stdin=subprocess.PIPE, stdout=subprocess.PIPE, shell=False)
    p.stdin.write(json.dumps(data).encode())
    p.stdin.close()
    
    return p;

def long_chain() -> tuple[float, int]:
    start_time = 0
    for i in range(tot_n):
        req = {
            "data_size": data_size * 1024 * 1024,
            "now_n": i+1,
            "tot_n": tot_n,
            "timestamp": start_time
        }
        handler = invoke_func(func_name, req)
        if i == 0:
            rsp = handler.stdout.read().decode()
            rsp_dict = json.loads(rsp)
            start_time = rsp_dict["timestamp_start"]
        if i == tot_n - 1:
            rsp = handler.stdout.read().decode()
            print(rsp)
            rsp_dict = json.loads(rsp)
            end_time = rsp_dict["timestamp_end"]
            output_size = rsp_dict["output_size"]
            return (end_time - start_time, output_size)


if __name__ == "__main__":
    cost_time, output_size = long_chain()

    cost_time_list = []
    for i in range(10):
        cost_time, _ = long_chain()
        cost_time_list.append(cost_time)
    
    cost_time_list.remove(max(cost_time_list))
    cost_time_list.remove(min(cost_time_list))
    tot_cost_time = sum(cost_time_list)

    print(f"Average cost time: {tot_cost_time/8}ms")
    #print(f"output size: {output_size}B")
