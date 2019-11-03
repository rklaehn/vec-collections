[0m[1m[38;5;9mabc::binary_merge::MergeOperation::merge0:
[0m [0m[1m[38;5;12mpush   [0m rbp
[0m [0m[1m[38;5;12mmov    [0m rbp, rsp
[0m [0m[1m[38;5;12mpush   [0m r15
[0m [0m[1m[38;5;12mpush   [0m r14
[0m [0m[1m[38;5;12mpush   [0m r13
[0m [0m[1m[38;5;12mpush   [0m r12
[0m [0m[1m[38;5;12mpush   [0m rbx
[0m [0m[1m[38;5;12msub    [0m rsp, 152
[0m [0m[1m[38;5;12mmov    [0m r14, rcx
[0m [0m[1m[38;5;12mmov    [0m r15, rsi
[0m [0m[1m[38;5;12mtest   [0m rdx, rdx
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_12
[0m [0m[1m[38;5;12mmov    [0m rbx, rdx
[0m [0m[1m[38;5;12mtest   [0m r14, r14
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_22
[0m [0m[1m[38;5;12mmov    [0m r8, rdi
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [r15, +, 16]
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [r15, +, 48]
[0m [0m[1m[38;5;12mmov    [0m rdx, rsi
[0m [0m[1m[38;5;12msub    [0m rdx, rdi
[0m [0m[1m[38;5;12mjb     [0m[1m[38;5;10m LBB4_112
[0m [0m[1m[38;5;12mmov    [0m r13, rbx
[0m [0m[1m[38;5;12mshr    [0m r13
[0m [0m[1m[38;5;12mcmp    [0m r13, rdx
[0m [0m[1m[38;5;12mjae    [0m[1m[38;5;10m LBB4_113
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [r15, +, 32]
[0m [0m[1m[38;5;12mcmp    [0m rsi, r14
[0m [0m[1m[38;5;12mjb     [0m[1m[38;5;10m LBB4_114
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 48], rbx
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [r15, +, 24]
[0m [0m[1m[38;5;12mshl    [0m rdi, 2
[0m [0m[1m[38;5;12madd    [0m rdi, qword, ptr, [r15]
[0m [0m[1m[38;5;12mmov    [0m ecx, dword, ptr, [rdi, +, 4*r13]
[0m [0m[1m[38;5;12mxor    [0m r12d, r12d
[0m [0m[1m[38;5;12mcmp    [0m r14, 1
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_10
[0m [0m[1m[38;5;12mmov    [0m rdx, r14
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_8
[0m[0m[1m[38;5;10mLBB4_7:
[0m [0m[1m[38;5;12msub    [0m rdx, rsi
[0m [0m[1m[38;5;12mcmp    [0m rdx, 1
[0m [0m[1m[38;5;12mjbe    [0m[1m[38;5;10m LBB4_10
[0m[0m[1m[38;5;10mLBB4_8:
[0m [0m[1m[38;5;12mmov    [0m rsi, rdx
[0m [0m[1m[38;5;12mshr    [0m rsi
[0m [0m[1m[38;5;12mlea    [0m rdi, [rsi, +, r12]
[0m [0m[1m[38;5;12mcmp    [0m ecx, dword, ptr, [rax, +, 4*rdi]
[0m [0m[1m[38;5;12mjb     [0m[1m[38;5;10m LBB4_7
[0m [0m[1m[38;5;12mmov    [0m r12, rdi
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_7
[0m[0m[1m[38;5;10mLBB4_10:
[0m [0m[1m[38;5;12mxor    [0m edx, edx
[0m [0m[1m[38;5;12mxor    [0m esi, esi
[0m [0m[1m[38;5;12mcmp    [0m ecx, dword, ptr, [rax, +, 4*r12]
[0m [0m[1m[38;5;12msetae  [0m sil
[0m [0m[1m[38;5;12mlea    [0m rax, [rsi, +, rsi, -, 1]
[0m [0m[1m[38;5;12mcmovne [0m rdx, rax
[0m [0m[1m[38;5;12mcmp    [0m rdx, -1
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 160], r14
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_24
[0m [0m[1m[38;5;12mmov    [0m bl, 1
[0m [0m[1m[38;5;12mtest   [0m rdx, rdx
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_25
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_26
[0m[0m[1m[38;5;10mLBB4_12:
[0m [0m[1m[38;5;12mtest   [0m r14, r14
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_107
[0m [0m[1m[38;5;12mmov    [0m rcx, qword, ptr, [r15, +, 40]
[0m [0m[1m[38;5;12mmov    [0m rbx, qword, ptr, [r15, +, 48]
[0m [0m[1m[38;5;12mmov    [0m rax, rbx
[0m [0m[1m[38;5;12msub    [0m rax, rcx
[0m [0m[1m[38;5;12mcmp    [0m rax, r14
[0m [0m[1m[38;5;12mjae    [0m[1m[38;5;10m LBB4_101
[0m [0m[1m[38;5;12mmov    [0m r13, qword, ptr, [r15, +, 16]
[0m [0m[1m[38;5;12mcmp    [0m r13, rbx
[0m [0m[1m[38;5;12mjb     [0m[1m[38;5;10m LBB4_119
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [r15, +, 32]
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [r15, +, 16], rbx
[0m [0m[1m[38;5;12mmov    [0m r8, qword, ptr, [r15]
[0m [0m[1m[38;5;12mlea    [0m rax, [r8, +, 4*rbx]
[0m [0m[1m[38;5;12mmov    [0m r12, r13
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 144], rbx
[0m [0m[1m[38;5;12msub    [0m r12, rbx
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 136], r12
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 128], rax
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 120], rax
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 112], r15
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 104], rdi
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [rbp, -, 96], 0
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 48], rdi
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_45
[0m [0m[1m[38;5;12mtest   [0m rdi, rdi
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_50
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [r15, +, 8]
[0m [0m[1m[38;5;12mmov    [0m rax, rsi
[0m [0m[1m[38;5;12msub    [0m rax, r13
[0m [0m[1m[38;5;12mcmp    [0m rax, rdi
[0m [0m[1m[38;5;12mjae    [0m[1m[38;5;10m LBB4_54
[0m [0m[1m[38;5;12madd    [0m r13, rdi
[0m [0m[1m[38;5;12mjb     [0m[1m[38;5;10m LBB4_121
[0m [0m[1m[38;5;12mlea    [0m rax, [rsi, +, rsi]
[0m [0m[1m[38;5;12mcmp    [0m r13, rax
[0m [0m[1m[38;5;12mcmovb  [0m r13, rax
[0m [0m[1m[38;5;12mmov    [0m edx, 4
[0m [0m[1m[38;5;12mxor    [0m ecx, ecx
[0m [0m[1m[38;5;12mmov    [0m rax, r13
[0m [0m[1m[38;5;12mmul    [0m rdx
[0m [0m[1m[38;5;12msetno  [0m dl
[0m [0m[1m[38;5;12mjo     [0m[1m[38;5;10m LBB4_121
[0m [0m[1m[38;5;12mmov    [0m cl, dl
[0m [0m[1m[38;5;12mshl    [0m rcx, 2
[0m [0m[1m[38;5;12mtest   [0m rsi, rsi
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 56], rcx
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 152], rax
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_52
[0m [0m[1m[38;5;12mshl    [0m rsi, 2
[0m [0m[1m[38;5;12mmov    [0m edx, 4
[0m [0m[1m[38;5;12mmov    [0m rdi, r8
[0m [0m[1m[38;5;12mmov    [0m rcx, rax
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m ___rust_realloc
[0m [0m[1m[38;5;12mmov    [0m r8, rax
[0m [0m[1m[38;5;12mtest   [0m rax, rax
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 48]
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_126
[0m[0m[1m[38;5;10mLBB4_53:
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [r15], r8
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [r15, +, 8], r13
[0m[0m[1m[38;5;10mLBB4_54:
[0m [0m[1m[38;5;12mlea    [0m r13, [rdi, +, rbx]
[0m [0m[1m[38;5;12mlea    [0m rsi, [r8, +, 4*rbx]
[0m [0m[1m[38;5;12mlea    [0m rdi, [r8, +, 4*r13]
[0m [0m[1m[38;5;12mshl    [0m r12, 2
[0m [0m[1m[38;5;12mmov    [0m rdx, r12
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m _memmove
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 48]
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 144], r13
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [r15, +, 16]
[0m [0m[1m[38;5;12mmov    [0m rbx, rdi
[0m [0m[1m[38;5;12mcmp    [0m rax, r13
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_58
[0m [0m[1m[38;5;12mmov    [0m rcx, qword, ptr, [r15]
[0m [0m[1m[38;5;12mshl    [0m rax, 2
[0m [0m[1m[38;5;12mshl    [0m r13, 2
[0m [0m[1m[38;5;12mmov    [0m rbx, rdi
[0m[0m[1m[38;5;10mLBB4_56:
[0m [0m[1m[38;5;12mtest   [0m rbx, rbx
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_94
[0m [0m[1m[38;5;12mdec    [0m rbx
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 104], rbx
[0m [0m[1m[38;5;12mmov    [0m edx, dword, ptr, [rbp, -, 96]
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [rcx, +, rax], edx
[0m [0m[1m[38;5;12minc    [0m qword, ptr, [r15, +, 16]
[0m [0m[1m[38;5;12mmov    [0m rbx, qword, ptr, [rbp, -, 104]
[0m [0m[1m[38;5;12madd    [0m rax, 4
[0m [0m[1m[38;5;12mcmp    [0m r13, rax
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_56
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_58
[0m[0m[1m[38;5;10mLBB4_22:
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [r15, +, 40]
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [r15, +, 48]
[0m [0m[1m[38;5;12mcmp    [0m rsi, rax
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_39
[0m [0m[1m[38;5;12mmovq   [0m xmm0, rsi
[0m [0m[1m[38;5;12mpshufd [0m xmm0, xmm0, 68
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_44
[0m[0m[1m[38;5;10mLBB4_24:
[0m [0m[1m[38;5;12mxor    [0m edx, edx
[0m[0m[1m[38;5;10mLBB4_25:
[0m [0m[1m[38;5;12madd    [0m r12, rdx
[0m [0m[1m[38;5;12mxor    [0m ebx, ebx
[0m[0m[1m[38;5;10mLBB4_26:
[0m [0m[1m[38;5;12mmov    [0m r14, r8
[0m [0m[1m[38;5;12mmov    [0m rdi, r8
[0m [0m[1m[38;5;12mmov    [0m rsi, r15
[0m [0m[1m[38;5;12mmov    [0m rdx, r13
[0m [0m[1m[38;5;12mmov    [0m rcx, r12
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m abc::binary_merge::MergeOperation::merge0
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [r15, +, 40]
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [r15, +, 48]
[0m [0m[1m[38;5;12mmovq   [0m xmm0, rax
[0m [0m[1m[38;5;12mpshufd [0m xmm0, xmm0, 68
[0m [0m[1m[38;5;12mtest   [0m bl, bl
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_33
[0m [0m[1m[38;5;12mcmp    [0m rax, rsi
[0m [0m[1m[38;5;12mmov    [0m rbx, qword, ptr, [rbp, -, 160]
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_31
[0m [0m[1m[38;5;12mmov    [0m rdx, qword, ptr, [r15, +, 16]
[0m [0m[1m[38;5;12mcmp    [0m rdx, rax
[0m [0m[1m[38;5;12mjbe    [0m[1m[38;5;10m LBB4_117
[0m [0m[1m[38;5;12mcmp    [0m rdx, rsi
[0m [0m[1m[38;5;12mjbe    [0m[1m[38;5;10m LBB4_118
[0m [0m[1m[38;5;12mmov    [0m rcx, qword, ptr, [r15]
[0m [0m[1m[38;5;12mmov    [0m eax, dword, ptr, [rcx, +, 4*rax]
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [rcx, +, 4*rsi], eax
[0m [0m[1m[38;5;12mmovdqu [0m xmm0, xmmword, ptr, [r15, +, 40]
[0m[0m[1m[38;5;10mLBB4_31:
[0m [0m[1m[38;5;12mpcmpeqd[0m xmm1, xmm1
[0m [0m[1m[38;5;12mpsubq  [0m xmm0, xmm1
[0m [0m[1m[38;5;12mmovdqu [0m xmmword, ptr, [r15, +, 40], xmm0
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [r15, +, 32]
[0m [0m[1m[38;5;12mtest   [0m rax, rax
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_116
[0m [0m[1m[38;5;12mdec    [0m rax
[0m [0m[1m[38;5;12madd    [0m qword, ptr, [r15, +, 24], 4
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [r15, +, 32], rax
[0m [0m[1m[38;5;12mnot    [0m r13
[0m [0m[1m[38;5;12madd    [0m r13, qword, ptr, [rbp, -, 48]
[0m [0m[1m[38;5;12mnot    [0m r12
[0m [0m[1m[38;5;12madd    [0m r12, rbx
[0m [0m[1m[38;5;12mmov    [0m rdi, r14
[0m [0m[1m[38;5;12mmov    [0m rsi, r15
[0m [0m[1m[38;5;12mmov    [0m rdx, r13
[0m [0m[1m[38;5;12mmov    [0m rcx, r12
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_38
[0m[0m[1m[38;5;10mLBB4_33:
[0m [0m[1m[38;5;12mcmp    [0m rax, rsi
[0m [0m[1m[38;5;12mmov    [0m rcx, qword, ptr, [rbp, -, 160]
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_37
[0m [0m[1m[38;5;12mmov    [0m rdx, qword, ptr, [r15, +, 16]
[0m [0m[1m[38;5;12mcmp    [0m rdx, rax
[0m [0m[1m[38;5;12mjbe    [0m[1m[38;5;10m LBB4_117
[0m [0m[1m[38;5;12mcmp    [0m rdx, rsi
[0m [0m[1m[38;5;12mjbe    [0m[1m[38;5;10m LBB4_118
[0m [0m[1m[38;5;12mmov    [0m rdx, qword, ptr, [r15]
[0m [0m[1m[38;5;12mmov    [0m eax, dword, ptr, [rdx, +, 4*rax]
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [rdx, +, 4*rsi], eax
[0m [0m[1m[38;5;12mmovdqu [0m xmm0, xmmword, ptr, [r15, +, 40]
[0m[0m[1m[38;5;10mLBB4_37:
[0m [0m[1m[38;5;12mpcmpeqd[0m xmm1, xmm1
[0m [0m[1m[38;5;12mpsubq  [0m xmm0, xmm1
[0m [0m[1m[38;5;12mmovdqu [0m xmmword, ptr, [r15, +, 40], xmm0
[0m [0m[1m[38;5;12mnot    [0m r13
[0m [0m[1m[38;5;12madd    [0m r13, qword, ptr, [rbp, -, 48]
[0m [0m[1m[38;5;12msub    [0m rcx, r12
[0m [0m[1m[38;5;12mmov    [0m rdi, r14
[0m [0m[1m[38;5;12mmov    [0m rsi, r15
[0m [0m[1m[38;5;12mmov    [0m rdx, r13
[0m[0m[1m[38;5;10mLBB4_38:
[0m [0m[1m[38;5;12madd    [0m rsp, 152
[0m [0m[1m[38;5;12mpop    [0m rbx
[0m [0m[1m[38;5;12mpop    [0m r12
[0m [0m[1m[38;5;12mpop    [0m r13
[0m [0m[1m[38;5;12mpop    [0m r14
[0m [0m[1m[38;5;12mpop    [0m r15
[0m [0m[1m[38;5;12mpop    [0m rbp
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m __ZN3abc12binary_merge14MergeOperation6merge017hf74b6d7251a73ba3E
[0m[0m[1m[38;5;10mLBB4_39:
[0m [0m[1m[38;5;12mmov    [0m rcx, rbx
[0m[0m[1m[38;5;10mLBB4_40:
[0m [0m[1m[38;5;12mmov    [0m rdx, qword, ptr, [r15, +, 16]
[0m [0m[1m[38;5;12mcmp    [0m rdx, rsi
[0m [0m[1m[38;5;12mjbe    [0m[1m[38;5;10m LBB4_111
[0m [0m[1m[38;5;12mcmp    [0m rdx, rax
[0m [0m[1m[38;5;12mjbe    [0m[1m[38;5;10m LBB4_109
[0m [0m[1m[38;5;12mmov    [0m rdx, qword, ptr, [r15]
[0m [0m[1m[38;5;12mmov    [0m edi, dword, ptr, [rdx, +, 4*rsi]
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [rdx, +, 4*rax], edi
[0m [0m[1m[38;5;12minc    [0m rax
[0m [0m[1m[38;5;12minc    [0m rsi
[0m [0m[1m[38;5;12mdec    [0m rcx
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_40
[0m [0m[1m[38;5;12mmovdqu [0m xmm0, xmmword, ptr, [r15, +, 40]
[0m[0m[1m[38;5;10mLBB4_44:
[0m [0m[1m[38;5;12mmovq   [0m xmm1, rbx
[0m [0m[1m[38;5;12mpshufd [0m xmm1, xmm1, 68
[0m [0m[1m[38;5;12mpaddq  [0m xmm1, xmm0
[0m [0m[1m[38;5;12mmovdqu [0m xmmword, ptr, [r15, +, 40], xmm1
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_107
[0m[0m[1m[38;5;10mLBB4_45:
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [r15, +, 8]
[0m [0m[1m[38;5;12mmov    [0m rax, rsi
[0m [0m[1m[38;5;12msub    [0m rax, rbx
[0m [0m[1m[38;5;12mcmp    [0m rax, rdi
[0m [0m[1m[38;5;12mjae    [0m[1m[38;5;10m LBB4_51
[0m [0m[1m[38;5;12madd    [0m rbx, rdi
[0m [0m[1m[38;5;12mjb     [0m[1m[38;5;10m LBB4_122
[0m [0m[1m[38;5;12mlea    [0m rax, [rsi, +, rsi]
[0m [0m[1m[38;5;12mcmp    [0m rbx, rax
[0m [0m[1m[38;5;12mcmovb  [0m rbx, rax
[0m [0m[1m[38;5;12mmov    [0m ecx, 4
[0m [0m[1m[38;5;12mxor    [0m r12d, r12d
[0m [0m[1m[38;5;12mmov    [0m rax, rbx
[0m [0m[1m[38;5;12mmul    [0m rcx
[0m [0m[1m[38;5;12mmov    [0m r13, rax
[0m [0m[1m[38;5;12msetno  [0m al
[0m [0m[1m[38;5;12mjo     [0m[1m[38;5;10m LBB4_122
[0m [0m[1m[38;5;12mmov    [0m r12b, al
[0m [0m[1m[38;5;12mshl    [0m r12, 2
[0m [0m[1m[38;5;12mtest   [0m rsi, rsi
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_72
[0m [0m[1m[38;5;12mshl    [0m rsi, 2
[0m [0m[1m[38;5;12mmov    [0m edx, 4
[0m [0m[1m[38;5;12mmov    [0m rdi, r8
[0m [0m[1m[38;5;12mmov    [0m rcx, r13
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m ___rust_realloc
[0m [0m[1m[38;5;12mmov    [0m r8, rax
[0m [0m[1m[38;5;12mtest   [0m rax, rax
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 48]
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_73
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_120
[0m[0m[1m[38;5;10mLBB4_50:
[0m [0m[1m[38;5;12mxor    [0m ebx, ebx
[0m[0m[1m[38;5;10mLBB4_58:
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 88], 4
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 80], 0
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 72], 0
[0m [0m[1m[38;5;12mtest   [0m rbx, rbx
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_64
[0m [0m[1m[38;5;12mmov    [0m ecx, 4
[0m [0m[1m[38;5;12mxor    [0m r12d, r12d
[0m [0m[1m[38;5;12mmov    [0m rax, rbx
[0m [0m[1m[38;5;12mmul    [0m rcx
[0m [0m[1m[38;5;12mmov    [0m r13, rax
[0m [0m[1m[38;5;12msetno  [0m al
[0m [0m[1m[38;5;12mjo     [0m[1m[38;5;10m LBB4_123
[0m [0m[1m[38;5;12mmov    [0m r12b, al
[0m [0m[1m[38;5;12mshl    [0m r12, 2
[0m [0m[1m[38;5;12mmov    [0m rdi, r13
[0m [0m[1m[38;5;12mmov    [0m rsi, r12
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m ___rust_alloc
[0m [0m[1m[38;5;12mtest   [0m rax, rax
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_120
[0m [0m[1m[38;5;12mmov    [0m rcx, rax
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 88], rax
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 80], rbx
[0m [0m[1m[38;5;12mxor    [0m r12d, r12d
[0m[0m[1m[38;5;10mLBB4_62:
[0m [0m[1m[38;5;12mdec    [0m rbx
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 104], rbx
[0m [0m[1m[38;5;12mmov    [0m eax, dword, ptr, [rbp, -, 96]
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [rcx, +, 4*r12], eax
[0m [0m[1m[38;5;12minc    [0m r12
[0m [0m[1m[38;5;12mmov    [0m rbx, qword, ptr, [rbp, -, 104]
[0m [0m[1m[38;5;12mtest   [0m rbx, rbx
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_62
[0m [0m[1m[38;5;12mmov    [0m r13, qword, ptr, [rbp, -, 88]
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [rbp, -, 80]
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 48]
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_65
[0m[0m[1m[38;5;10mLBB4_64:
[0m [0m[1m[38;5;12mmov    [0m r13d, 4
[0m [0m[1m[38;5;12mxor    [0m eax, eax
[0m [0m[1m[38;5;12mxor    [0m r12d, r12d
[0m[0m[1m[38;5;10mLBB4_65:
[0m [0m[1m[38;5;12mlea    [0m rbx, [r13, +, 4*r12]
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 88], r13
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 80], rax
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 72], r13
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 64], rbx
[0m [0m[1m[38;5;12mtest   [0m r12, r12
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_90
[0m [0m[1m[38;5;12mshl    [0m r12, 2
[0m [0m[1m[38;5;12msar    [0m r12, 2
[0m [0m[1m[38;5;12mmov    [0m r8, qword, ptr, [rbp, -, 112]
[0m [0m[1m[38;5;12mmov    [0m r9, qword, ptr, [rbp, -, 144]
[0m [0m[1m[38;5;12mmov    [0m rcx, qword, ptr, [rbp, -, 136]
[0m [0m[1m[38;5;12mlea    [0m rdi, [rcx, +, r9]
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [r8, +, 8]
[0m [0m[1m[38;5;12mmov    [0m rax, rsi
[0m [0m[1m[38;5;12msub    [0m rax, rdi
[0m [0m[1m[38;5;12mcmp    [0m rax, r12
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 56], r8
[0m [0m[1m[38;5;12mjae    [0m[1m[38;5;10m LBB4_71
[0m [0m[1m[38;5;12madd    [0m rdi, r12
[0m [0m[1m[38;5;12mjb     [0m[1m[38;5;10m LBB4_124
[0m [0m[1m[38;5;12mlea    [0m rax, [rsi, +, rsi]
[0m [0m[1m[38;5;12mcmp    [0m rdi, rax
[0m [0m[1m[38;5;12mcmovb  [0m rdi, rax
[0m [0m[1m[38;5;12mmov    [0m edx, 4
[0m [0m[1m[38;5;12mxor    [0m r10d, r10d
[0m [0m[1m[38;5;12mmov    [0m rax, rdi
[0m [0m[1m[38;5;12mmul    [0m rdx
[0m [0m[1m[38;5;12msetno  [0m dl
[0m [0m[1m[38;5;12mjo     [0m[1m[38;5;10m LBB4_124
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 184], rdi
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 192], r9
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 152], rcx
[0m [0m[1m[38;5;12mmov    [0m r10b, dl
[0m [0m[1m[38;5;12mshl    [0m r10, 2
[0m [0m[1m[38;5;12mtest   [0m rsi, rsi
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 176], r10
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 168], rax
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_82
[0m [0m[1m[38;5;12mshl    [0m rsi, 2
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [r8]
[0m [0m[1m[38;5;12mmov    [0m edx, 4
[0m [0m[1m[38;5;12mmov    [0m rcx, rax
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m ___rust_realloc
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_83
[0m[0m[1m[38;5;10mLBB4_51:
[0m [0m[1m[38;5;12mtest   [0m rdi, rdi
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_74
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_79
[0m[0m[1m[38;5;10mLBB4_71:
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [r8]
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_85
[0m[0m[1m[38;5;10mLBB4_52:
[0m [0m[1m[38;5;12mmov    [0m rdi, rax
[0m [0m[1m[38;5;12mmov    [0m rsi, rcx
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m ___rust_alloc
[0m [0m[1m[38;5;12mmov    [0m r8, rax
[0m [0m[1m[38;5;12mtest   [0m rax, rax
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 48]
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_53
[0m[0m[1m[38;5;10mLBB4_126:
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 152]
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [rbp, -, 56]
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m alloc::alloc::handle_alloc_error
[0m[0m[1m[38;5;10mLBB4_72:
[0m [0m[1m[38;5;12mmov    [0m rdi, r13
[0m [0m[1m[38;5;12mmov    [0m rsi, r12
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m ___rust_alloc
[0m [0m[1m[38;5;12mmov    [0m r8, rax
[0m [0m[1m[38;5;12mtest   [0m rax, rax
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 48]
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_120
[0m[0m[1m[38;5;10mLBB4_73:
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [r15], r8
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [r15, +, 8], rbx
[0m [0m[1m[38;5;12mmov    [0m rbx, qword, ptr, [r15, +, 16]
[0m[0m[1m[38;5;10mLBB4_74:
[0m [0m[1m[38;5;12mmov    [0m rax, rdi
[0m [0m[1m[38;5;12mdec    [0m rax
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 104], rax
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [r8, +, 4*rbx], 0
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_80
[0m [0m[1m[38;5;12mmov    [0m rax, rdi
[0m [0m[1m[38;5;12madd    [0m rax, -2
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 104], rax
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [r8, +, 4*rbx, +, 4], 0
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_81
[0m [0m[1m[38;5;12mmov    [0m rax, rdi
[0m [0m[1m[38;5;12madd    [0m rax, -3
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 104], rax
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [r8, +, 4*rbx, +, 8], 0
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_108
[0m [0m[1m[38;5;12mmov    [0m rax, rdi
[0m [0m[1m[38;5;12madd    [0m rax, -4
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 104], rax
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [r8, +, 4*rbx, +, 12], 0
[0m [0m[1m[38;5;12mlea    [0m rbx, [rbx, +, 4]
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_79
[0m[0m[1m[38;5;10mLBB4_78:
[0m [0m[1m[38;5;12mmov    [0m ecx, dword, ptr, [rbp, -, 96]
[0m [0m[1m[38;5;12mdec    [0m rax
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 104], rax
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [r8, +, 4*rbx], ecx
[0m [0m[1m[38;5;12minc    [0m rbx
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [rbp, -, 104]
[0m [0m[1m[38;5;12mtest   [0m rax, rax
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_78
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_79
[0m[0m[1m[38;5;10mLBB4_80:
[0m [0m[1m[38;5;12minc    [0m rbx
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_79
[0m[0m[1m[38;5;10mLBB4_81:
[0m [0m[1m[38;5;12madd    [0m rbx, 2
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_79
[0m[0m[1m[38;5;10mLBB4_82:
[0m [0m[1m[38;5;12mmov    [0m rdi, rax
[0m [0m[1m[38;5;12mmov    [0m rsi, r10
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m ___rust_alloc
[0m[0m[1m[38;5;10mLBB4_83:
[0m [0m[1m[38;5;12mtest   [0m rax, rax
[0m [0m[1m[38;5;12mmov    [0m rcx, qword, ptr, [rbp, -, 152]
[0m [0m[1m[38;5;12mmov    [0m r9, qword, ptr, [rbp, -, 192]
[0m [0m[1m[38;5;12mmov    [0m rdx, qword, ptr, [rbp, -, 184]
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_127
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [rbp, -, 56]
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rsi], rax
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rsi, +, 8], rdx
[0m[0m[1m[38;5;10mLBB4_85:
[0m [0m[1m[38;5;12madd    [0m r12, r9
[0m [0m[1m[38;5;12mlea    [0m rsi, [rax, +, 4*r9]
[0m [0m[1m[38;5;12mlea    [0m rdi, [rax, +, 4*r12]
[0m [0m[1m[38;5;12mshl    [0m rcx, 2
[0m [0m[1m[38;5;12mmov    [0m rdx, rcx
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m _memmove
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 144], r12
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [rbp, -, 56]
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [rsi, +, 16]
[0m [0m[1m[38;5;12mcmp    [0m rax, r12
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_87
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 48]
[0m [0m[1m[38;5;12mcmp    [0m r13, rbx
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_91
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_92
[0m[0m[1m[38;5;10mLBB4_87:
[0m [0m[1m[38;5;12mmov    [0m rcx, qword, ptr, [rsi]
[0m [0m[1m[38;5;12mshl    [0m rax, 2
[0m [0m[1m[38;5;12mshl    [0m r12, 2
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 48]
[0m[0m[1m[38;5;10mLBB4_88:
[0m [0m[1m[38;5;12mcmp    [0m r13, rbx
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_92
[0m [0m[1m[38;5;12mlea    [0m rdx, [r13, +, 4]
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 72], rdx
[0m [0m[1m[38;5;12mmov    [0m edx, dword, ptr, [r13]
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [rcx, +, rax], edx
[0m [0m[1m[38;5;12minc    [0m qword, ptr, [rsi, +, 16]
[0m [0m[1m[38;5;12mmov    [0m r13, qword, ptr, [rbp, -, 72]
[0m [0m[1m[38;5;12mmov    [0m rbx, qword, ptr, [rbp, -, 64]
[0m [0m[1m[38;5;12madd    [0m rax, 4
[0m [0m[1m[38;5;12mcmp    [0m r12, rax
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_88
[0m[0m[1m[38;5;10mLBB4_90:
[0m [0m[1m[38;5;12mcmp    [0m r13, rbx
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_92
[0m[0m[1m[38;5;10mLBB4_91:
[0m [0m[1m[38;5;12msub    [0m rbx, r13
[0m [0m[1m[38;5;12madd    [0m rbx, -4
[0m [0m[1m[38;5;12mand    [0m rbx, -4
[0m [0m[1m[38;5;12mlea    [0m rax, [rbx, +, r13, +, 4]
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 72], rax
[0m[0m[1m[38;5;10mLBB4_92:
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [rbp, -, 80]
[0m [0m[1m[38;5;12mtest   [0m rsi, rsi
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_94
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 88]
[0m [0m[1m[38;5;12mshl    [0m rsi, 2
[0m [0m[1m[38;5;12mmov    [0m edx, 4
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m ___rust_dealloc
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 48]
[0m[0m[1m[38;5;10mLBB4_94:
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [rbp, -, 128]
[0m [0m[1m[38;5;12mmov    [0m rcx, qword, ptr, [rbp, -, 120]
[0m [0m[1m[38;5;12mcmp    [0m rax, rcx
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_96
[0m[0m[1m[38;5;10mLBB4_95:
[0m [0m[1m[38;5;12msub    [0m rcx, rax
[0m [0m[1m[38;5;12madd    [0m rcx, -4
[0m [0m[1m[38;5;12mand    [0m rcx, -4
[0m [0m[1m[38;5;12mlea    [0m rax, [rcx, +, rax, +, 4]
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [rbp, -, 128], rax
[0m[0m[1m[38;5;10mLBB4_96:
[0m [0m[1m[38;5;12mmov    [0m r13, qword, ptr, [rbp, -, 136]
[0m [0m[1m[38;5;12mtest   [0m r13, r13
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_100
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [rbp, -, 144]
[0m [0m[1m[38;5;12mmov    [0m r12, qword, ptr, [rbp, -, 112]
[0m [0m[1m[38;5;12mmov    [0m rbx, qword, ptr, [r12, +, 16]
[0m [0m[1m[38;5;12mcmp    [0m rax, rbx
[0m [0m[1m[38;5;12mje     [0m[1m[38;5;10m LBB4_99
[0m [0m[1m[38;5;12mmov    [0m rcx, qword, ptr, [r12]
[0m [0m[1m[38;5;12mlea    [0m rsi, [rcx, +, 4*rax]
[0m [0m[1m[38;5;12mlea    [0m rdi, [rcx, +, 4*rbx]
[0m [0m[1m[38;5;12mlea    [0m rdx, [4*r13]
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m _memmove
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 48]
[0m[0m[1m[38;5;10mLBB4_99:
[0m [0m[1m[38;5;12madd    [0m rbx, r13
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [r12, +, 16], rbx
[0m[0m[1m[38;5;10mLBB4_100:
[0m [0m[1m[38;5;12madd    [0m qword, ptr, [r15, +, 48], rdi
[0m [0m[1m[38;5;12mmov    [0m rcx, qword, ptr, [r15, +, 40]
[0m[0m[1m[38;5;10mLBB4_101:
[0m [0m[1m[38;5;12mlea    [0m rdi, [4*rcx]
[0m [0m[1m[38;5;12mxor    [0m esi, esi
[0m[0m[1m[38;5;10mLBB4_102:
[0m [0m[1m[38;5;12mmov    [0m rdx, qword, ptr, [r15, +, 32]
[0m [0m[1m[38;5;12mcmp    [0m rsi, rdx
[0m [0m[1m[38;5;12mjae    [0m[1m[38;5;10m LBB4_110
[0m [0m[1m[38;5;12mlea    [0m rax, [rcx, +, rsi]
[0m [0m[1m[38;5;12mmov    [0m rdx, qword, ptr, [r15, +, 16]
[0m [0m[1m[38;5;12mcmp    [0m rdx, rax
[0m [0m[1m[38;5;12mjbe    [0m[1m[38;5;10m LBB4_109
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [r15, +, 24]
[0m [0m[1m[38;5;12mmov    [0m eax, dword, ptr, [rax, +, 4*rsi]
[0m [0m[1m[38;5;12mmov    [0m rdx, qword, ptr, [r15]
[0m [0m[1m[38;5;12madd    [0m rdx, rdi
[0m [0m[1m[38;5;12mmov    [0m dword, ptr, [rdx, +, 4*rsi], eax
[0m [0m[1m[38;5;12mlea    [0m rax, [rsi, +, 1]
[0m [0m[1m[38;5;12mmov    [0m rsi, rax
[0m [0m[1m[38;5;12mcmp    [0m r14, rax
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_102
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [r15, +, 32]
[0m [0m[1m[38;5;12mmov    [0m rax, rsi
[0m [0m[1m[38;5;12msub    [0m rax, r14
[0m [0m[1m[38;5;12mjb     [0m[1m[38;5;10m LBB4_115
[0m [0m[1m[38;5;12mlea    [0m rcx, [4*r14]
[0m [0m[1m[38;5;12madd    [0m qword, ptr, [r15, +, 24], rcx
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [r15, +, 32], rax
[0m [0m[1m[38;5;12madd    [0m qword, ptr, [r15, +, 40], r14
[0m[0m[1m[38;5;10mLBB4_107:
[0m [0m[1m[38;5;12madd    [0m rsp, 152
[0m [0m[1m[38;5;12mpop    [0m rbx
[0m [0m[1m[38;5;12mpop    [0m r12
[0m [0m[1m[38;5;12mpop    [0m r13
[0m [0m[1m[38;5;12mpop    [0m r14
[0m [0m[1m[38;5;12mpop    [0m r15
[0m [0m[1m[38;5;12mpop    [0m rbp
[0m [0m[1m[38;5;12mret[0m
[0m[0m[1m[38;5;10mLBB4_108:
[0m [0m[1m[38;5;12madd    [0m rbx, 3
[0m[0m[1m[38;5;10mLBB4_79:
[0m [0m[1m[38;5;12mmov    [0m qword, ptr, [r15, +, 16], rbx
[0m [0m[1m[38;5;12mmov    [0m rax, qword, ptr, [rbp, -, 128]
[0m [0m[1m[38;5;12mmov    [0m rcx, qword, ptr, [rbp, -, 120]
[0m [0m[1m[38;5;12mcmp    [0m rax, rcx
[0m [0m[1m[38;5;12mjne    [0m[1m[38;5;10m LBB4_95
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_96
[0m[0m[1m[38;5;10mLBB4_109:
[0m [0m[1m[38;5;12mlea    [0m rdi, [rip, +, l_anon.110dcd1db61b8b64f7d8b146edfe9a6a.4]
[0m [0m[1m[38;5;12mmov    [0m rsi, rax
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::panicking::panic_bounds_check
[0m[0m[1m[38;5;10mLBB4_110:
[0m [0m[1m[38;5;12mlea    [0m rdi, [rip, +, l_anon.110dcd1db61b8b64f7d8b146edfe9a6a.6]
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::panicking::panic_bounds_check
[0m[0m[1m[38;5;10mLBB4_111:
[0m [0m[1m[38;5;12mlea    [0m rdi, [rip, +, l_anon.110dcd1db61b8b64f7d8b146edfe9a6a.3]
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::panicking::panic_bounds_check
[0m[0m[1m[38;5;10mLBB4_112:
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::slice::slice_index_order_fail
[0m[0m[1m[38;5;10mLBB4_113:
[0m [0m[1m[38;5;12mlea    [0m rdi, [rip, +, l_anon.110dcd1db61b8b64f7d8b146edfe9a6a.5]
[0m [0m[1m[38;5;12mmov    [0m rsi, r13
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::panicking::panic_bounds_check
[0m[0m[1m[38;5;10mLBB4_114:
[0m [0m[1m[38;5;12mmov    [0m rdi, r14
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::slice::slice_index_len_fail
[0m[0m[1m[38;5;10mLBB4_115:
[0m [0m[1m[38;5;12mmov    [0m rdi, r14
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::slice::slice_index_order_fail
[0m[0m[1m[38;5;10mLBB4_116:
[0m [0m[1m[38;5;12mmov    [0m edi, 1
[0m [0m[1m[38;5;12mxor    [0m esi, esi
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::slice::slice_index_order_fail
[0m[0m[1m[38;5;10mLBB4_117:
[0m [0m[1m[38;5;12mlea    [0m rdi, [rip, +, l_anon.110dcd1db61b8b64f7d8b146edfe9a6a.3]
[0m [0m[1m[38;5;12mmov    [0m rsi, rax
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::panicking::panic_bounds_check
[0m[0m[1m[38;5;10mLBB4_118:
[0m [0m[1m[38;5;12mlea    [0m rdi, [rip, +, l_anon.110dcd1db61b8b64f7d8b146edfe9a6a.4]
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::panicking::panic_bounds_check
[0m[0m[1m[38;5;10mLBB4_119:
[0m [0m[1m[38;5;12mlea    [0m rdi, [rip, +, l_anon.110dcd1db61b8b64f7d8b146edfe9a6a.2]
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::panicking::panic
[0m[0m[1m[38;5;10mLBB4_120:
[0m [0m[1m[38;5;12mmov    [0m rdi, r13
[0m [0m[1m[38;5;12mmov    [0m rsi, r12
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m alloc::alloc::handle_alloc_error
[0m[0m[1m[38;5;10mLBB4_121:
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m alloc::raw_vec::capacity_overflow
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_125
[0m[0m[1m[38;5;10mLBB4_122:
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m alloc::raw_vec::capacity_overflow
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_125
[0m[0m[1m[38;5;10mLBB4_123:
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m alloc::raw_vec::capacity_overflow
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_125
[0m[0m[1m[38;5;10mLBB4_124:
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m alloc::raw_vec::capacity_overflow
[0m[0m[1m[38;5;10mLBB4_125:
[0m [0m[1m[38;5;12mud2[0m
[0m[0m[1m[38;5;10mLBB4_127:
[0m [0m[1m[38;5;12mmov    [0m rdi, qword, ptr, [rbp, -, 168]
[0m [0m[1m[38;5;12mmov    [0m rsi, qword, ptr, [rbp, -, 176]
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m alloc::alloc::handle_alloc_error
[0m[0m[1m[38;5;10mLBB4_128:
[0m [0m[1m[38;5;12mmov    [0m rbx, rax
[0m [0m[1m[38;5;12mlea    [0m rdi, [rbp, -, 88]
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::ptr::real_drop_in_place
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_131
[0m[0m[1m[38;5;10mLBB4_129:
[0m [0m[1m[38;5;12mmov    [0m rbx, rax
[0m [0m[1m[38;5;12mlea    [0m rdi, [rbp, -, 88]
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::ptr::real_drop_in_place
[0m [0m[1m[38;5;12mjmp    [0m[1m[38;5;10m LBB4_131
[0m[0m[1m[38;5;10mLBB4_130:
[0m [0m[1m[38;5;12mmov    [0m rbx, rax
[0m[0m[1m[38;5;10mLBB4_131:
[0m [0m[1m[38;5;12mlea    [0m rdi, [rbp, -, 144]
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m core::ptr::real_drop_in_place
[0m [0m[1m[38;5;12mmov    [0m rdi, rbx
[0m [0m[1m[38;5;12mcall   [0m[1m[38;5;9m __Unwind_Resume
[0m [0m[1m[38;5;12mud2[0m
