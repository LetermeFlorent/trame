import sys

COLORS = 7
out = sys.argv[1]
count = int(sys.argv[2])
with open(out + "/flood-plain.txt", "w") as plain:
    for i in range(count):
        plain.write("line %d abcdefghijklmnopqrstuvwxyz 0123456789 abcdefghijklmnopqrstuvwxyz\n" % i)
with open(out + "/flood-color.txt", "w") as colored:
    for i in range(count):
        color = i % COLORS + 1
        colored.write(
            "\x1b[3%dm line %d \x1b[1;4%dm abcdefghijklmnopqrstuvwxyz \x1b[0m 0123456789 abcdefghijklmnopqrstuvwxyz\n"
            % (color, i, color)
        )
