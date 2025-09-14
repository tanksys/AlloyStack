#!/usr/bin/env python3
import re
from collections import Counter

# 读取fake_data_0.txt文件
file_path = './image_content/fake_data_0.txt'

with open(file_path, 'r') as f:
    content = f.read()

# 提取所有单词（去除标点符号）
words = re.findall(r'\b\w+\b', content.lower())

# 统计唯一单词数量
unique_words = set(words)
unique_word_count = len(unique_words)

# 找到最长单词及其长度
longest_word = max(unique_words, key=len) if unique_words else ''
longest_word_length = len(longest_word)

# 统计单词总数量
total_word_count = len(words)

# 分析单词频率分布
word_freq = Counter(words)

# 计算平均单词长度
avg_word_length = sum(len(word) for word in words) / total_word_count if total_word_count > 0 else 0

# 打印分析结果
print("===== fake_data_0.txt 分析结果 =====")
print(f"总单词数: {total_word_count}")
print(f"唯一单词数: {unique_word_count}")
print(f"最长单词: '{longest_word}' (长度: {longest_word_length})")
print(f"平均单词长度: {avg_word_length:.2f}")
print("\n===== 推荐参数值 =====")
print("# 基于fake_data_0.txt的分析，考虑到可能有多个文件和更大的数据量，建议以下参数值：")
print(f"MAX_WORD_LENGTH = {max(longest_word_length, 20)}  # 最长单词长度，保留一定余量")
print(f"MAX_WORDS = {max(unique_word_count * 2, 10000)}  # 唯一单词数量的2倍，确保足够空间")
print(f"MAX_SLOT_NUM = 100  # 通常设置为mapper或reducer的最大数量")
print(f"MAX_BUFFER_SIZE = {max(unique_word_count * 50, 100000)}  # 每个单词约占用50字节空间")