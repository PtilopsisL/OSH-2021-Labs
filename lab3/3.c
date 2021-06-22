#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/socket.h>
#include <sys/select.h>
#include <sys/time.h>
#include <netinet/in.h>

#define MAX_BUFFER_SIZE 1200000
#define MAX_USERS 32

int fds[MAX_USERS] = {0};
int fd_stat[MAX_USERS] = {0};
int used_len = 0;

const char msg[] = "Message:";
const char err_full[] = "server full, connection close.\n";
char *buffers[MAX_USERS];
ssize_t buff_len[MAX_USERS];
int counter[MAX_USERS];

int main(int argc, char **argv) {
    int port = atoi(argv[1]);
    int fd;
    if ((fd = socket(AF_INET, SOCK_STREAM, 0)) == 0) {
        perror("socket");
        return 1;
    }
    fcntl(fd, F_SETFL, fcntl(fd, F_GETFL, 0) | O_NONBLOCK);
    struct sockaddr_in addr;
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(port);
    socklen_t addr_len = sizeof(addr);
    if (bind(fd, (struct sockaddr *)&addr, sizeof(addr))) {
        perror("bind");
        return 1;
    }
    if (listen(fd, MAX_USERS)) {
        perror("listen");
        return 1;
    }
    printf("server start, max user: %d, current user: %d\n", MAX_USERS, used_len);

    fd_set clients;
    while (1) {
        int max_fd = fd;
        FD_ZERO(&clients);
        FD_SET(fd, &clients);
        for (int i = 0; i < MAX_USERS; i++) {
            if (fd_stat[i] == 1) {
                FD_SET(fds[i], &clients);
                max_fd = fds[i] > max_fd ? fds[i] : max_fd;
            }
        }
        if (select(max_fd + 1, &clients, NULL, NULL, NULL) > 0) {
            if (FD_ISSET(fd, &clients)) {
                int new_fd = accept(fd, NULL, NULL);
                if (new_fd == -1) {
                    perror("accept");
                    return 1;
                }

                if (used_len >= MAX_USERS) {
                    send(new_fd, err_full, sizeof(err_full) - 1, 0);
                    close(new_fd);
                    printf("users queue full!\n");
                    goto end;
                }

                int empty_fd;
                for (empty_fd = 0; fd_stat[empty_fd] == 1 && empty_fd < MAX_USERS; empty_fd++);
                fds[empty_fd] = new_fd;
                fd_stat[empty_fd] = 1;
                used_len++;
                printf("new user connected, current user: %d\n", used_len);
                buffers[empty_fd] = (char *)malloc(MAX_BUFFER_SIZE);
                memcpy(buffers[empty_fd], msg, 8);
                buff_len[empty_fd] = 8;
                counter[empty_fd] = 8;
            }
end:
            for (int i = 0; i < MAX_USERS; i++) {
                if (fd_stat[i] && FD_ISSET(fds[i], &clients)){
                    ssize_t len;
                    while ((len = recv(fds[i], buffers[i] + buff_len[i], 1024, 0)) > 0) {
                        buff_len[i] += len;
                        while (counter[i] < buff_len[i]){
                            while (buffers[i][counter[i]] != '\n' && counter[i] < buff_len[i]){
                                counter[i]++;
                            }

                            if (counter[i] < buff_len[i]){
                                for (int j = 0; j < MAX_USERS; j++){
                                    if (fd_stat[j] == 1 && fds[j] != fds[i]) {
                                        int send_head = 0;
                                        while (send_head != counter[i] + 1) {
                                            int send_len = send(fds[j], buffers[i] + send_head, counter[i] + 1 - send_head, 0);
                                            if (send_len < 0) {
                                                perror("send");
                                                exit(0);
                                            }
                                            send_head += send_len;
                                        }
                                    }
                                }
                                
                                memcpy(buffers[i] + 8, buffers[i] + counter[i] + 1, buff_len[i] - counter[i] - 1);
                                buff_len[i] -= (counter[i] - 7);
                                counter[i] = 8;
                            }
                        }
                    }
                    if (len == 0) {
                        fd_stat[i] = 0;
                        used_len--;
                        free(buffers[i]);
                        printf("one user disconnected, current user: %d\n", used_len);
                    }
                }
            }
        }
    }
    return 0;
}
