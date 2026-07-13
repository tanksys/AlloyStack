__version__ = "minimal-0.1"


class _DType:
    def __init__(self, name):
        self.name = name

    def __repr__(self):
        return f"dtype('{self.name}')"


int64 = _DType("int64")
float64 = _DType("float64")


class ndarray:
    def __init__(self, data, shape=None, dtype=None):
        self._data = list(data)
        self.dtype = dtype
        self.shape = tuple(shape) if shape is not None else (len(self._data),)

    def reshape(self, *shape):
        if len(shape) == 1 and isinstance(shape[0], (tuple, list)):
            shape = tuple(shape[0])
        size = 1
        for dim in shape:
            size *= dim
        if size != len(self._data):
            raise ValueError("cannot reshape array of size {} into shape {}".format(
                len(self._data), shape
            ))
        return ndarray(self._data, shape, self.dtype)

    def sum(self):
        return sum(self._data)

    def tolist(self):
        if len(self.shape) == 1:
            return list(self._data)
        if len(self.shape) == 2:
            rows, cols = self.shape
            return [
                self._data[row * cols:(row + 1) * cols]
                for row in range(rows)
            ]
        return list(self._data)

    def __add__(self, other):
        return ndarray([value + other for value in self._data], self.shape, self.dtype)

    def __radd__(self, other):
        return self.__add__(other)

    def __iter__(self):
        return iter(self.tolist())

    def __len__(self):
        return self.shape[0]

    def __repr__(self):
        return "array({})".format(self.tolist())


def array(data, dtype=None):
    if isinstance(data, ndarray):
        return ndarray(data._data, data.shape, dtype or data.dtype)
    if data and isinstance(data[0], (list, tuple)):
        rows = len(data)
        cols = len(data[0])
        flat = []
        for row in data:
            if len(row) != cols:
                raise ValueError("ragged nested sequences are not supported")
            flat.extend(row)
        return ndarray(flat, (rows, cols), dtype)
    return ndarray(data, dtype=dtype)


def arange(*args, dtype=None):
    if len(args) == 1:
        start, stop, step = 0, args[0], 1
    elif len(args) == 2:
        start, stop = args
        step = 1
    elif len(args) == 3:
        start, stop, step = args
    else:
        raise TypeError("arange expected 1 to 3 positional arguments")
    return ndarray(range(start, stop, step), dtype=dtype)


def sum(values):
    if isinstance(values, ndarray):
        return values.sum()
    total = 0
    for value in values:
        total += value
    return total
