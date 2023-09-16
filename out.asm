section .text
	global _start
_start:
	mov rax, 5
	push rax
	push QWORD [rsp + 0]
	mov rax, 2
	push rax
	pop rax
	pop rbx
	add rax, rbx
	push rax
	push QWORD [rsp + 8]
	push QWORD [rsp + 8]
	pop rax
	pop rbx
	add rax, rbx
	push rax
	mov rax, 60
	pop rdi
	syscall
