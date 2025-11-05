#include <assert.h>
#include <errno.h>
#include <setjmp.h>
#include <signal.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/ucontext.h>
#include <unistd.h>

#include "wasmtime-platform.h"


static wasmtime_trap_handler_t g_handler = NULL;

static void handle_signal(int signo, siginfo_t *info, void *context)
{
    assert(g_handler != NULL);
    uintptr_t ip, fp;
#if defined(__aarch64__)
    ucontext_t *cx = context;
    ip = cx->uc_mcontext.pc;
    fp = cx->uc_mcontext.regs[29];
#elif defined(__x86_64__)
    ucontext_t *cx = context;
    ip = cx->uc_mcontext.gregs[REG_RIP];
    fp = cx->uc_mcontext.gregs[REG_RBP];
#else
#error "Unsupported platform"
#endif

    bool has_faulting_addr = signo == SIGSEGV;
    uintptr_t faulting_addr = 0;
    if (has_faulting_addr)
        faulting_addr = (uintptr_t)info->si_addr;
    g_handler(ip, fp, has_faulting_addr, faulting_addr);

    // If wasmtime didn't handle this trap then reset the handler to the default
    // behavior which will probably abort the process.
    signal(signo, SIG_DFL);
}

int wasmtime_init_traps(wasmtime_trap_handler_t handler)
{
    int rc;
    g_handler = handler;

    struct sigaction action;
    memset(&action, 0, sizeof(action));

    action.sa_sigaction = handle_signal;
    action.sa_flags = SA_SIGINFO | SA_NODEFER;
    sigemptyset(&action.sa_mask);

    rc = sigaction(SIGILL, &action, NULL);
    if (rc != 0)
        return errno;
    rc = sigaction(SIGSEGV, &action, NULL);
    if (rc != 0)
        return errno;
    rc = sigaction(SIGFPE, &action, NULL);
    if (rc != 0)
        return errno;
    return 0;
}