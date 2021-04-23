# ba codec

repeat of count, byte, count, byte...

count = 0bABBNNNNN

- `A` = repeat or skip (`0` = repeat, `1` = skip)
- `BB` = count bytes count (0 ~ 3) (1=1~32, 1=33~8192, 2=8193~2097152, 3=invalid)

