// SentientOS Linux syscall translation layer
// Provides compatibility with Linux syscalls
#ifndef SENTIENT_SYSCALL_TRANSLATOR_H
#define SENTIENT_SYSCALL_TRANSLATOR_H

// Syscall wrapper for ZK verification
struct zk_syscall_context {
    unsigned long syscall_number;
    void* args[6];
    int verified;
    char hash[65];
};

// Function prototypes
int sentient_syscall_init();
int sentient_syscall_exec(struct zk_syscall_context* ctx);
int sentient_syscall_verify(const char* hash);

#endif
