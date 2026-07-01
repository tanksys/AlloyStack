#include <Python.h>
#include <stdio.h>
#include <string.h>

int bcmp(const void *left, const void *right, size_t size)
{
    return memcmp(left, right, size);
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

    status = PyConfig_SetString(&config, &config.program_name, L"python");
    if (report_status(status))
        goto init_failed;
    status = PyConfig_SetString(&config, &config.home, L"/");
    if (report_status(status))
        goto init_failed;
    status = PyWideStringList_Append(&config.module_search_paths, L"Lib");
    if (report_status(status))
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
