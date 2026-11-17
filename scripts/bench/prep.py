import sys
d = sys.argv[1]
n = int(sys.argv[2])
with open(d + "/flood-plain.txt", "w") as f:
    for i in range(n):
        f.write("line %d abcdefghijklmnopqrstuvwxyz 0123456789 abcdefghijklmnopqrstuvwxyz\n" % i)
with open(d + "/flood-color.txt", "w") as f:
    for i in range(n):
        f.write("\x1b[3%dm line %d \x1b[1;4%dm abcdefghijklmnopqrstuvwxyz \x1b[0m 0123456789 abcdefghijklmnopqrstuvwxyz\n" % (i % 7 + 1, i, i % 7 + 1))
