#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <pthread.h>

#define MAX_BUFFER_SIZE 1200000

const char msg[] = "Message:";

struct Pipe {
    int fd_send;
    int fd_recv;
};

void *handle_chat(void *data) {
    struct Pipe *pipe = (struct Pipe *)data;
    char* buffer = (char *)malloc(MAX_BUFFER_SIZE);
    memcpy(buffer, msg, 8);
    ssize_t buff_len = 8;
    ssize_t len;
    int counter = 8;
    while ((len = recv(pipe->fd_send, buffer + buff_len, 1024, 0)) > 0) {
        buff_len += len;
        while (counter < buff_len){
            while (buffer[counter] != '\n' && counter < buff_len){
                counter++;
            }

            if (counter < buff_len){
                int send_head = 0;
                while (send_head != counter + 1) {
                    int send_len = send(pipe->fd_recv, buffer + send_head, counter + 1 - send_head, 0);
                    if (send_len < 0) {
                        perror("send");
                        exit(0);
                    }
                    send_head += send_len;
                }
                
                memcpy(buffer + 8, buffer + counter + 1, buff_len - counter - 1);
                buff_len -= (counter - 7);
                counter = 8;
            }
        }
    }
    free(buffer);
    return NULL;
}

int main(int argc, char **argv) {
    int port = atoi(argv[1]);
    int fd;
    if ((fd = socket(AF_INET, SOCK_STREAM, 0)) == 0) {
        perror("socket");
        return 1;
    }
    struct sockaddr_in addr;
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(port);
    socklen_t addr_len = sizeof(addr);
    if (bind(fd, (struct sockaddr *)&addr, sizeof(addr))) {
        perror("bind");
        return 1;
    }
    if (listen(fd, 2)) {
        perror("listen");
        return 1;
    }
    int fd1 = accept(fd, NULL, NULL);
    int fd2 = accept(fd, NULL, NULL);
    if (fd1 == -1 || fd2 == -1) {
        perror("accept");
        return 1;
    }
    pthread_t thread1, thread2;
    struct Pipe pipe1;
    struct Pipe pipe2;
    pipe1.fd_send = fd1;
    pipe1.fd_recv = fd2;
    pipe2.fd_send = fd2;
    pipe2.fd_recv = fd1;
    pthread_create(&thread1, NULL, handle_chat, (void *)&pipe1);
    pthread_create(&thread2, NULL, handle_chat, (void *)&pipe2);
    pthread_join(thread1, NULL);
    pthread_join(thread2, NULL);
    return 0;
}
