#include <stdio.h>
#include <string.h>

#include "../../target/x86_64-unknown-linux-musl/release/libpmem.h"

int main(int argc, char **argv)
{
	int ret;

	if (argc < 2) {
		fprintf(stderr, "Usage: %s path/to/linpmem.ko\n",
			argc ? argv[0] : "load_unload");
		return -1;
	}

	ret = pmem_load(argv[1]);
	if (ret) {
		fprintf(stderr, "error: %s\n", strerror(ret));
		return -1;
	}

	printf("Driver was loaded from %s. Check /dev for device file. Press any key to unload.\n",
	       argv[1]);
	getchar();

	ret = pmem_unload();
	if (ret) {
		fprintf(stderr, "error: %s\n", strerror(ret));
		return -1;
	}

	return 0;
}
