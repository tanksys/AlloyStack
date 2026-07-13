import numpy as np
print(f"numpy v{np.__version__} imported", flush=True)

def main():
    a = np.array([1, 2, 3, 4, 5])
    b = np.array([10, 20, 30, 40, 50])

    print("a =", a)
    print("b =", b)

    print("a + b =", a + b)
    print("a * b =", a * b)

    matrix = np.array([
        [1, 2, 3],
        [4, 5, 6]
    ])

    print("matrix:")
    print(matrix)

    print("transpose:")
    print(matrix.T)

    x = np.array([
        [1, 2],
        [3, 4]
    ])

    y = np.array([
        [5, 6],
        [7, 8]
    ])

    print("matrix multiplication:")
    print(x @ y)

    data = np.array([1, 2, 3, 4, 5])

    print("mean:", np.mean(data))
    print("sum:", np.sum(data))
    print("max:", np.max(data))
    return

if __name__ == "__main__":
    main()
