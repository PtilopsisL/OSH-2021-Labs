#include <stdio.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/sysmacros.h>
#include <sys/wait.h>
#include <sys/reboot.h>
#include <linux/reboot.h>

int main() {
    if (mknod("/dev/ttyS0", S_IFCHR | S_IRUSR | S_IWUSR, makedev(4, 64)) == -1) {
        perror("mknod() failed");
    }
    if (mknod("/dev/ttyAMA0", S_IFCHR | S_IRUSR | S_IWUSR, makedev(204, 64)) == -1) {
        perror("mknod() failed");
    }
    if (mknod("/dev/fb0", S_IFCHR | S_IRUSR | S_IWUSR, makedev(29, 0)) == -1) {
        perror("mknod() failed");
    }

    for (int i = 1; i <= 3; i++){
	    pid_t fpid = fork();
	    if (fpid == 0){
		    if (i == 1){
			    execl("./1", "1", NULL);
		    }
		    if (i == 2){
			    execl("./2", "2", NULL);
		    }
		    if (i == 3){
			    execl("./3", "3", NULL);
		    }
	    }
	    waitpid(fpid, NULL, 0);
	    printf("******Finish running %d! Sleep 2 seconds.******\n", i);
	    sleep(2);
    }
    printf("Finish!\n");
    sync();
    reboot(RB_HALT_SYSTEM);
}
