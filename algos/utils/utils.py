import math

def partition(array, a, b, p):
    i = a-1
    j = b
    while True:
        i += 1
        while i < b and array[i] < array[p]:
            i += 1
        j -= 1
        while j >= a and array[j] > array[p]:
            j -= 1
        if i < j:
            array[i].swap(array[j])
        else:
            return j


def medianOfThree(array, a, b):
    b -= 1
    m = a+(b-a)//2
    if array[a] > array[m]:
        array[a].swap(array[m])
    if array[m] > array[b]:
        array[m].swap(array[b])
        if array[a] > array[m]:
            return
    array[a].swap(array[m])


def medianOfThreeIndices(array, indices):
    if len(indices) == 0:
        return -1
    if len(indices) < 3:
        return indices[0]
    if array[indices[1]] > array[indices[0]]:
        if array[indices[2]] > array[indices[1]]:
            return indices[1]
        if array[indices[0]] < array[indices[2]]:
            return indices[2]
        return indices[0]
    if array[indices[2]] < array[indices[1]]:
        return indices[1]
    if array[indices[2]] > array[indices[0]]:
        return indices[0]
    return indices[2]


def medianOfThreeIdx(array, a, m, b):
    return medianOfThreeIndices(array, [a, m, b])


def medianOf9(array, a, b):
    l = b-a
    h = l//2
    q = h//2
    e = q//2
    m0 = medianOfThreeIndices(array, [a, a+e, a+q])
    m1 = medianOfThreeIndices(array, [a+q+e, a+h, a+h+e])
    m2 = medianOfThreeIndices(array, [a+h+q, a+h+q+e, b-1])
    return medianOfThreeIndices(array, [m0, m1, m2])


def mOMHelper(array, a, len):
    if len == 1:
        return a
    t = len//3
    return medianOfThreeIndices(array, [mOMHelper(array, a, t), mOMHelper(array, a+t, t), mOMHelper(array, a+2*t, t)])


def medianOfMedians(array, a, len):
    if len == 1:
        return a
    nearPow = 3**round(math.log(len, 3))
    if nearPow == len:
        return mOMHelper(array, a, len)
    nearPow //= 2
    if 2*nearPow >= len:
        nearPow //= 3
    return medianOfThreeIndices(array, [mOMHelper(array, a, nearPow), mOMHelper(array, a+len-nearPow, nearPow), medianOfMedians(array, a+nearPow, len-2*nearPow)])


def blockSwap(array, a, b, len):
    i = 0
    while i < len:
        array[a+i].swap(array[b+i])
        i += 1


def backwardBlockSwap(array, a, b, len):
    i = len-1
    while i >= 0:
        array[a+i].swap(array[b+i])
        i -= 1


def compareValues(a, b):
    return (a > b)-(a < b)

compareIntToValue = compareValues

def insertToLeft(array, _from, to):
    temp: Value
    idx: int
    temp, idx = array[_from].readNoMark()
    i = _from-1
    while i >= to:
        array[i+1].write(array[i].noMark())
        i -= 1
    array[to].writeRestoreIdx(temp, idx)


def insertToRight(array, _from, to):
    temp: Value
    idx: int
    temp, idx = array[_from].readNoMark()
    i = _from
    while i < to:
        array[i].write(array[i+1].noMark())
        i += 1
    array[to].writeRestoreIdx(temp, idx)


def checkMergeBounds(array, a, m, b, rotate):
    if rotate is None:
        rotate = sortingVisualizer.getRotationByName("Helium").indexed
    if array[m-1] <= array[m]:
        return True
    elif array[a] > array[b-1]:
        rotate(array, a, m, b)
        return True
    return False


def lrBinarySearch(array, a, b, val, left):
    while a < b:
        m = a+((b-a)//2)
        cmp: int
        cmp = compareValues(array[m], val)
        if (cmp >= 0 if left else cmp > 0):
            b = m
        else:
            a = m+1
    return a


def dualPivotPartition(array, a, b):
    l: int
    g: int
    k: int
    l = a+1
    g = b-1
    k = l
    while k <= g:
        if array[k] < array[a]:
            array[k].swap(array[l])
            l += 1
        elif array[k] >= array[b]:
            while array[g] > array[b] and k < g:
                g -= 1
            array[k].swap(array[g])
            g -= 1
            if array[k] < array[a]:
                array[k].swap(array[l])
                l += 1
        k += 1
    l -= 1
    g += 1
    array[a].swap(array[l])
    array[b].swap(array[g])
    return l, g


def LLPartition(array, a, b):
    b -= 1
    pivot = array[b].copy()
    i = a
    j = a
    while j <= b:
        if array[j] < pivot:
            array[i].swap(array[j])
            i += 1
        j += 1
    array[i].swap(array[b])
    return i


def findMinMaxIndices(array, a, b):
    currMin = a
    currMax = a
    i = a+1
    while i < b:
        if array[i] < array[currMin]:
            currMin = i
        elif array[i] > array[currMax]:
            currMax = i
        i += 1
    return currMin, currMax


def findMinMaxValue(array, a, b):
    min_: int
    max_: int
    min_, max_ = findMinMaxIndices(array, a, b)
    return array[min_].copy(), array[max_].copy()


def findMinMax(array, a, b):
    currMin: Value
    currMax: Value
    currMin, currMax = findMinMaxValue(array, a, b)
    return currMin.readInt(), currMax.readInt()


def findMinValue(array, a, b):
    currMin = array[a].copy()
    i = a+1
    while i < b:
        if array[i] < currMin:
            currMin = array[i].copy()
        i += 1
    return currMin


def findMin(array, a, b):
    return findMinValue(array, a, b).readInt()


def findHighestPower(array, a, b, base):
    return int(math.log(findMax(array, a, b), base))


def log2(n):
    target = 0
    while n != 0:
        n >>= 1
        target += 1
    return target


def javaNumberOfLeadingZeros(i):
    if i <= 0:
        return 32 if i == 0 else 0
    n = 31
    if i >= 1 << 16:
        n -= 16
        i >>= 16
    if i >= 1 << 8:
        n -= 8
        i >>= 8
    if i >= 1 << 4:
        n -= 4
        i >>= 4
    if i >= 1 << 2:
        n -= 2
        i >>= 2
    return n-(i >> 1)


def javaNumberOfTrailingZeros(i):
    i = ~i & (i-1)
    if i <= 0:
        return i & 32
    n = 1
    if i > 1 << 16:
        n += 16
        i >>= 16
    if i > 1 << 8:
        n += 8
        i >>= 8
    if i > 1 << 4:
        n += 4
        i >>= 4
    if i > 1 << 2:
        n += 2
        i >>= 2
    return n+(i >> 1)


def shift(array, pos, to):
    if to-pos > 0:
        i = pos
        while i < to:
            array[i].swap(array[i+1])
            i += 1
    else:
        i = pos
        while i > to:
            array[i].swap(array[i-1])
            i -= 1
