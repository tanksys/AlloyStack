#!python3

import subprocess
import sys
import json

func_name = "data-rw-rust"
gateway = "http://192.168.1.189:32222"
data_size = int(sys.argv[1])

def invoke_func(func_name: str, data: dict,) -> subprocess.Popen:
    p = subprocess.Popen(["faas-cli", "-g", gateway, "invoke", func_name], stdin=subprocess.PIPE, stdout=subprocess.PIPE, shell=False)
    p.stdin.write(json.dumps(data).encode())
    p.stdin.close()
    
    return p;

if __name__ == "__main__":
    req = {"data_size": data_size}   
    cost_time_list = []
    for i in range(10):
        handler = invoke_func(func_name, req)
        res = handler.stdout.read().decode()
        cost_time = json.loads(res)["cost_time"]
        cost_time_list.append(cost_time)

    cost_time_list.remove(max(cost_time_list))
    cost_time_list.remove(min(cost_time_list))
    
    print(f"cost time: {sum(cost_time_list)/8}ms")
