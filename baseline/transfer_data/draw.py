import matplotlib.pyplot as plt
import matplotlib as mpl
import numpy as np
import math
import matplotlib.font_manager

matplotlib.rcParams["font.family"] = 'Arial'
matplotlib.rcParams['pdf.fonttype'] = 42
matplotlib.rcParams['ps.fonttype'] = 42
plt.rcParams.update({'font.size': 20})
# 线宽
linewidth = 0.8
mpl.rcParams['hatch.linewidth'] = linewidth
fig = plt.figure(figsize=(9.3, 5), dpi=150)

# 数据
categories = ['1KB' , '64KB', '128KB']
vm_socket  = [1747.2, 2116.4, 2258.4 ]
proc_sock  = [218.5 , 249.1 , 257.3  ]
mmap       = [146.7 , 156.5 , 182.6  ]
func_call  = [0.2   , 6.4   , 13.1   ]

# 绘制柱状图
bar_width = 0.2
index = range(len(categories))

back_colors = ["#CED8E0", "#FFF6DB", "#F9F0FD",
               "#D5E8D4", "white", "white", "white"]

plt.bar(index, vm_socket, bar_width,
        color=back_colors[0], label='Inter-VM TCP', edgecolor="#000000", zorder=2, linewidth=linewidth,)
plt.bar([i + bar_width for i in index],
        proc_sock, bar_width, color=back_colors[1], label='Inter-Process TCP', edgecolor="#000000", zorder=2, linewidth=linewidth,)
plt.bar([i + 2 * bar_width for i in index],
        mmap, bar_width, color=back_colors[2], label='Shared Memory', edgecolor="#000000", zorder=2, linewidth=linewidth,)
plt.bar([i + 3 * bar_width for i in index],
        func_call, bar_width, color=back_colors[3], label='Function Call', edgecolor="#000000", zorder=2, linewidth=linewidth,)

# 在柱子中心位置显示数据
for i in index:
    plt.text(i + 0 * bar_width, 10 ** (math.log10(vm_socket[i])/4),
             str(vm_socket[i]), ha='center', rotation=90)
    plt.text(i + 1 * bar_width, 10 ** (math.log10(proc_sock[i])/4),
             str(proc_sock[i]), ha='center', rotation=90)
    plt.text(i + 2 * bar_width, 10 ** (math.log10(mmap[i])/4),
             str(mmap[i]), ha='center', rotation=90)
    plt.text(i + 3 * bar_width, 10 ** (math.log10(func_call[i])/4),
             str(func_call[i]), ha='center', rotation=90)

plt.xlabel('Data Size')
plt.ylabel('Time (us)')
# plt.title('Performance Comparison')
plt.xticks([i + 1.5 * bar_width for i in index], categories)
plt.yscale('log')  # 设置纵坐标为对数坐标轴
plt.legend(framealpha=0, loc=(0.07, 1.01), ncol=2)
plt.tight_layout()
plt.grid(ls="--", zorder=1)
plt.savefig('/home/yjn/Downloads/sec2_transfer.pdf')
# plt.show()
