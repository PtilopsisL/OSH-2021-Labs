#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <pthread.h>

#define MAX_BUFFER_SIZE 1200000
#define MAX_USERS 32

int fd;
int fds[MAX_USERS] = {0};
int fd_stat[MAX_USERS] = {0};
pthread_t threads[MAX_USERS];
int ready = 1;
int used_len = 0;

pthread_mutex_t mutex = PTHREAD_MUTEX_INITIALIZER;
pthread_cond_t cv = PTHREAD_COND_INITIALIZER;

const char msg[] = "Message:";
const char err_full[] = "server full, connection close.\n";

void *handle_chat(void *p) {
    int num = *(int *)p;
    free(p);

    pthread_mutex_lock(&mutex);
    while (!ready) {
        pthread_cond_wait(&cv, &mutex);
    }
    int curr_fd = fds[num];
    pthread_mutex_unlock(&mutex);

    char* buffer = (char *)malloc(MAX_BUFFER_SIZE);
    memcpy(buffer, msg, 8);
    ssize_t buff_len = 8;
    ssize_t len;
    int counter = 8;
    while ((len = recv(curr_fd, buffer + buff_len, 1024, 0)) > 0) {
        buff_len += len;
        while (buffer[counter] != '\n' && counter < buff_len){
            counter++;
        }

        pthread_mutex_lock(&mutex);
        while (!ready) {
            pthread_cond_wait(&cv, &mutex);
        }
        
        if (counter < buff_len){
            for (int i = 0; i < MAX_USERS; i++) {
                if (fd_stat[i] == 1 && fds[i] != curr_fd) {
                    if (send(fds[i], buffer, counter + 1, 0) != counter + 1) {
                        perror("send");
                        exit(0);
                    }
                }
            }
            
            memcpy(buffer + 8, buffer + counter + 1, buff_len - counter - 1);
            buff_len -= (counter - 7);
            counter = 8;
        }

        pthread_mutex_unlock(&mutex);
    }
    free(buffer);

    ready = 0;
    pthread_mutex_lock(&mutex);

    fd_stat[num] = 0;
    used_len--;

    ready = 1;
    pthread_cond_signal(&cv);
    pthread_mutex_unlock(&mutex);
    return NULL;
}

void *handle_accept(void *p){
    while (1){
        int new_fd = accept(fd, NULL, NULL);
        if (new_fd == -1){
            perror("accept");
            exit(0);
        }

        ready = 0;
        pthread_mutex_lock(&mutex);
        
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
        int *ptr = (int *)malloc(sizeof(int));
        *ptr = empty_fd;
        pthread_create(&threads[empty_fd], NULL, handle_chat, (void *)ptr);
        used_len++;

end:
        ready = 1;
        pthread_cond_signal(&cv);
        pthread_mutex_unlock(&mutex);
    }
}

int main(int argc, char **argv) {
    int port = atoi(argv[1]);
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
    if (listen(fd, MAX_USERS)) {
        perror("listen");
        return 1;
    }
    handle_accept(NULL);
    return 0;
}
