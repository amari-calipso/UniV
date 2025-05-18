def pow2Sqrt(n):
    s = 1
    while s**2 < n:
        s *= 2
    return s


def checkSortedIdx(array, a, b):
    while a < b-1:
        if array[a] > array[a+1]:
            return a
        a += 1
    return b


def findKeysSorted(array, a, b, q):
    n = 1
    p = a
    i = a+1
    while i < b and n < q:
        if array[i] > array[i-1]:
            heliumRotate(array, p, p+n, i)
            p += i-(p+n)
            n += 1
        i += 1
    if n == q:
        heliumRotate(array, a, p, p+n)
    else:
        heliumRotate(array, p, p+n, b)
    return n


def findKeysUnsorted(array, a, p, b, q, to):
    n = p-a
    l: int
    p = a
    i = p+n
    while i < b and n < q:
        l = lrBinarySearch(array, p, p+n, array[i], True)
        if i == l or array[i] != array[l]:
            heliumRotate(array, p, p+n, i)
            l += i-(p+n)
            p += i-(p+n)
            insertToLeft(array, p+n, l)
            n += 1
        i += 1
    heliumRotate(array, to, p, p+n)
    return n


def findKeys(array, a, b, q, t):
    p = checkSortedIdx(array, a, b)
    if p == b:
        return -1
    if p-a <= t:
        return findKeysUnsorted(array, a, a, b, q, a)
    else:
        n = findKeysSorted(array, a, p, q)
        if n == q:
            return n
        return findKeysUnsorted(array, p-n, p, b, q, a)
