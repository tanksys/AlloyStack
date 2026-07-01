#include <Python.h>
#include <alloy/asbuffer.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int bcmp(const void *left, const void *right, size_t size)
{
    return memcmp(left, right, size);
}

static PyObject *pyas_error(const char *operation, int error)
{
    PyErr_Format(PyExc_RuntimeError, "%s failed: errno=%d", operation, -error);
    return NULL;
}

static PyObject *pyas_buffer_register(PyObject *self, PyObject *args)
{
    const char *slot;
    Py_ssize_t capacity;
    as_buffer_t buffer = AS_BUFFER_INIT;
    PyObject *view;
    int error;

    (void)self;
    if (!PyArg_ParseTuple(args, "sn:buffer_register", &slot, &capacity))
        return NULL;
    if (capacity < 0) {
        PyErr_SetString(PyExc_ValueError, "buffer capacity must be non-negative");
        return NULL;
    }

    error = as_buffer_alloc((size_t)capacity, &buffer);
    if (error)
        return pyas_error("as_buffer_alloc", error);
    memset(buffer.data, 0, (size_t)capacity);

    view = PyMemoryView_FromMemory((char *)buffer.data, capacity, PyBUF_WRITE);
    if (!view) {
        as_buffer_release(&buffer);
        return NULL;
    }

    /*
     * Publish before returning, matching the original WASM pyas semantics.
     * The returned memoryview is a non-owning producer view; synchronization
     * through a flag must happen after the producer finishes writing.
     */
    error = as_buffer_publish(slot, &buffer, (size_t)capacity);
    if (error) {
        Py_DECREF(view);
        if (buffer.data)
            as_buffer_release(&buffer);
        return pyas_error("as_buffer_publish", error);
    }
    return view;
}

static PyObject *pyas_access_buffer(PyObject *self, PyObject *args)
{
    const char *slot;
    PyObject *target;
    Py_buffer target_view;
    as_buffer_t source = AS_BUFFER_INIT;
    int error;

    (void)self;
    if (!PyArg_ParseTuple(args, "sO:access_buffer", &slot, &target))
        return NULL;
    if (PyObject_GetBuffer(target, &target_view, PyBUF_WRITABLE) < 0)
        return NULL;

    error = as_buffer_borrow(slot, &source);
    if (error == -ENOENT) {
        memset(target_view.buf, 0, (size_t)target_view.len);
        PyBuffer_Release(&target_view);
        Py_RETURN_NONE;
    }
    if (error) {
        PyBuffer_Release(&target_view);
        return pyas_error("as_buffer_take", error);
    }
    if ((size_t)target_view.len < source.len) {
        PyBuffer_Release(&target_view);
        PyErr_Format(PyExc_ValueError,
                     "target buffer is too small: need %zu, got %zd",
                     source.len, target_view.len);
        return NULL;
    }

    memcpy(target_view.buf, source.data, source.len);
    if ((size_t)target_view.len > source.len)
        memset((char *)target_view.buf + source.len, 0,
               (size_t)target_view.len - source.len);
    PyBuffer_Release(&target_view);
    Py_RETURN_NONE;
}

static PyObject *pyas_borrow_buffer(PyObject *self, PyObject *args)
{
    const char *slot;
    as_buffer_t source = AS_BUFFER_INIT;
    int error;

    (void)self;
    if (!PyArg_ParseTuple(args, "s:borrow_buffer", &slot))
        return NULL;
    error = as_buffer_borrow(slot, &source);
    if (error == -ENOENT)
        Py_RETURN_NONE;
    if (error)
        return pyas_error("as_buffer_borrow", error);
    return PyMemoryView_FromMemory((char *)source.data,
                                   (Py_ssize_t)source.len, PyBUF_READ);
}

static PyMethodDef pyas_methods[] = {
    {"buffer_register", pyas_buffer_register, METH_VARARGS,
     "Allocate and publish a writable shared buffer."},
    {"access_buffer", pyas_access_buffer, METH_VARARGS,
     "Borrow a shared buffer and copy it into a writable Python buffer."},
    {"borrow_buffer", pyas_borrow_buffer, METH_VARARGS,
     "Return a zero-copy, read-only view of a shared buffer."},
    {NULL, NULL, 0, NULL},
};

static struct PyModuleDef pyas_module = {
    PyModuleDef_HEAD_INIT,
    "pyas",
    "AlloyStack shared-buffer bindings.",
    -1,
    pyas_methods,
};

PyMODINIT_FUNC PyInit_pyas(void)
{
    return PyModule_Create(&pyas_module);
}

static const char *find_arg(int argc, char **argv, const char *key)
{
    size_t key_len = strlen(key);
    for (int i = 1; i < argc; ++i) {
        if (argv[i][0] == '-' && argv[i][1] == '-' &&
            strncmp(argv[i] + 2, key, key_len) == 0 &&
            argv[i][key_len + 2] == '=')
            return argv[i] + key_len + 3;
    }
    return NULL;
}

static int report_status(PyStatus status)
{
    if (!PyStatus_Exception(status))
        return 0;
    fprintf(stderr, "CPython init failed: %s\n",
            status.err_msg ? status.err_msg : "unknown error");
    return 1;
}

static int configure_argv(PyConfig *config, const char *script,
                          int argc, char **argv)
{
    const char *id = find_arg(argc, argv, "id");
    const char *mapper_num = find_arg(argc, argv, "mapper_num");
    const char *reducer_num = find_arg(argc, argv, "reducer_num");
    const char *sorter_num = find_arg(argc, argv, "sorter_num");
    const char *merger_num = find_arg(argc, argv, "merger_num");
    const char *func_num = find_arg(argc, argv, "func_num");
    char *python_argv[4];
    Py_ssize_t python_argc = 0;
    PyStatus status;

    python_argv[python_argc++] = (char *)(script ? script : "python");
    if (func_num) {
        python_argv[python_argc++] = (char *)func_num;
    } else if (sorter_num && merger_num) {
        python_argv[python_argc++] = (char *)(id ? id : "0");
        python_argv[python_argc++] = (char *)sorter_num;
        python_argv[python_argc++] = (char *)merger_num;
    } else if (mapper_num && reducer_num) {
        python_argv[python_argc++] = (char *)(id ? id : "0");
        python_argv[python_argc++] = (char *)mapper_num;
        python_argv[python_argc++] = (char *)reducer_num;
    }

    config->parse_argv = 0;
    status = PyConfig_SetBytesArgv(config, python_argc, python_argv);
    return report_status(status);
}

int main(int argc, char **argv)
{
    const char *script = find_arg(argc, argv, "pyfile_path");
    PyConfig config;
    PyStatus status;
    int rc = 0;

    PyConfig_InitIsolatedConfig(&config);
    config.install_signal_handlers = 0;
    config.site_import = 0;
    config.write_bytecode = 0;
    config.module_search_paths_set = 1;

    if (PyImport_AppendInittab("pyas", PyInit_pyas) < 0) {
        fprintf(stderr, "failed to register built-in pyas module\n");
        goto init_failed;
    }
    status = PyConfig_SetString(&config, &config.program_name, L"python");
    if (report_status(status))
        goto init_failed;
    status = PyConfig_SetString(&config, &config.home, L"/");
    if (report_status(status))
        goto init_failed;
    status = PyWideStringList_Append(&config.module_search_paths, L"Lib");
    if (report_status(status))
        goto init_failed;
    if (configure_argv(&config, script, argc, argv))
        goto init_failed;
    status = Py_InitializeFromConfig(&config);
    if (report_status(status))
        goto init_failed;
    PyConfig_Clear(&config);

    if (script) {
        FILE *file = fopen(script, "r");
        if (!file) {
            fprintf(stderr, "cannot open Python script: %s\n", script);
            rc = 2;
        } else {
            rc = PyRun_SimpleFileExFlags(file, script, 1, NULL);
        }
    } else {
        rc = PyRun_SimpleString("print('native musl CPython is running')");
    }
    /*
     * The LibOS currently exposes only one musl thread descriptor per host
     * thread. Py_FinalizeEx() deletes CPython's pthread keys and can block in
     * musl's process-wide key-table teardown. The isolation is unloaded after
     * this entry point returns, so defer finalization until that pthread
     * lifecycle is implemented by as_musl.
     */
    return rc;

init_failed:
    PyConfig_Clear(&config);
    return 1;
}
