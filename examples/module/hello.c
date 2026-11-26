// SPDX-License-Identifier: MIT
#include <linux/init.h>
#include <linux/module.h>

static int __init klab_hello_init(void)
{
	pr_info("klab_hello: loaded\n");
	return 0;
}

static void __exit klab_hello_exit(void)
{
	pr_info("klab_hello: unloaded\n");
}

module_init(klab_hello_init);
module_exit(klab_hello_exit);
MODULE_LICENSE("Dual MIT/GPL");
MODULE_DESCRIPTION("KERNEL//LAB educational logging example");
