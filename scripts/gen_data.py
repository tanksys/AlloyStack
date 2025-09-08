#!/usr/bin/python3
import os

workdir = "image_content"

# 固定随机种子，修改此值即可得到另一套稳定数据
SEED = 42


def _ensure_dir():
    if not os.path.isdir(workdir):
        os.makedirs(workdir, exist_ok=True)


def _write_exact_bytes(f, chunk: bytes, total_bytes: int):
    """
    将 chunk（字节模式）重复写入文件 f，直到精确写满 total_bytes 字节。
    最后一次不足 chunk 长度时写入切片。
    """
    written = 0
    clen = len(chunk)
    # 批量写
    while written + clen <= total_bytes:
        f.write(chunk)
        written += clen
    # 余数
    remain = total_bytes - written
    if remain:
        f.write(chunk[:remain])


def gen_word_count(file_num: int, total_size: int):
    """
    生成 word count 测试数据：
    - 每个文件大小可控，所有文件总字节数 = total_size
    - 文本内容可复现
    """
    from faker import Faker
    Faker.seed(SEED)
    fake = Faker()

    _ensure_dir()

    # 均分；最后一个文件补余数
    base_size = total_size // file_num
    sizes = [base_size] * file_num
    sizes[-1] += total_size - base_size * file_num

    # 构造一个稳定的文本块：一次生成，去掉换行，保证重复拼接后字节数稳定
    # 长度选择 10_000 附近（与原逻辑呼应），可适当增大/缩小
    raw_chunk = fake.text(10_000).replace('\n', ' ')
    # 确保非空
    if not raw_chunk:
        raw_chunk = "placeholder "
    # 转成 bytes
    chunk_bytes = raw_chunk.encode('utf-8')

    for i, target_size in enumerate(sizes):
        file_name = f"{workdir}/fake_data_{i}.txt"
        with open(file_name, "wb") as f:
            _write_exact_bytes(f, chunk_bytes, target_size)


def gen_parallel_sort(file_num: int, total_size: int):
    """
    生成 parallel sort 测试数据：
    - 内容为逗号分隔的整数序列 + 末尾 '1'
    - 保持末尾追加 '1' 的既有约定
    - 每个文件大小严格控制，所有文件总字节数 = total_size
      (注意：每个文件最后一个字节是字符 '1')
    """
    import random

    _ensure_dir()

    base_size = total_size // file_num
    sizes = [base_size] * file_num
    sizes[-1] += total_size - base_size * file_num

    for i, target_size in enumerate(sizes):
        # 预留最后 1 字节给 '1'
        if target_size == 0:
            # 极端情况保护
            body_size = 0
        else:
            body_size = max(0, target_size - 1)

        # 针对每个文件单独 seed，减少跨文件状态干扰
        rnd = random.Random(SEED + i)

        # 构造一个确定性的数字块（例如 10 个随机整数 + 逗号），重复使用
        nums = [str(rnd.randint(0, 1_000_000)) for _ in range(10)]
        block_str = ",".join(nums) + ","
        block_bytes = block_str.encode('utf-8')

        file_name = f"{workdir}/sort_data_{i}.txt"
        with open(file_name, "wb") as f:
            # 写主体 (body_size) 字节
            if body_size > 0:
                _write_exact_bytes(f, block_bytes, body_size)
            # 追加末尾 '1'
            f.write(b'1')
        # 可选：若需要校验，可在调试时加断言
        # assert os.path.getsize(file_name) == target_size, (file_name, os.path.getsize(file_name), target_size)


if __name__ == "__main__":
    import sys
    wc_args = 3, 100 * 1024 * 1024
    if len(sys.argv) == 5 and eval(sys.argv[1]) and eval(sys.argv[2]):
        wc_args = [eval(s) for s in sys.argv[1:3]]
        gen_word_count(*wc_args)

    ps_args = 3, 25 * 1024 * 1024
    if len(sys.argv) == 5 and eval(sys.argv[3]) and eval(sys.argv[4]):
        ps_args = [eval(s) for s in sys.argv[3:5]]
        gen_parallel_sort(*ps_args)