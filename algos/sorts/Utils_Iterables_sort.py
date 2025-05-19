class UtilsIterablesSortMerge:
    def __init__(this, size, aux):
        if aux is None:
            this.aux = sortingVisualizer.createValueArray(size)
        else:
            this.aux = aux
        this.__capacity = size

    def rotate(this, array, a, m, b):
        rl = b-m
        ll = m-a
        bl = 1 if this.aux is None else len(this.aux)
        min_ = bl if rl != ll and min(bl, rl, ll) > 8 else 1
        while ll > min_ and rl > min_:
            if rl < ll:
                blockSwap(array, a, m, rl)
                a += rl
                ll -= rl
            else:
                b -= ll
                rl -= ll
                backwardBlockSwap(array, a, b, ll)
        if rl == 1:
            insertToLeft(array, m, a)
        elif ll == 1:
            insertToRight(array, a, b-1)
        if min_ == 1 or rl <= 1 or ll <= 1:
            return
        if rl < ll:
            bidirArrayCopy(array, m, this.aux, 0, rl)
            bidirArrayCopy(array, a, array, b-ll, ll)
            bidirArrayCopy(this.aux, 0, array, a, rl)
        else:
            bidirArrayCopy(array, a, this.aux, 0, ll)
            bidirArrayCopy(array, m, array, a, rl)
            bidirArrayCopy(this.aux, 0, array, b-ll, ll)

    def __mergeUp(this, array, a, m, b):
        for i in range(m-a):
            this.aux[i].write(array[i+a])
        aux = 0
        left = a
        right = m
        while left < right and right < b:
            if this.aux[aux] <= array[right]:
                array[left].write(this.aux[aux])
                aux += 1
            else:
                array[left].write(array[right])
                right += 1
            left += 1
        while left < right:
            array[left].write(this.aux[aux])
            left += 1
            aux += 1

    def __mergeDown(this, array, a, m, b):
        for i in range(b-m):
            this.aux[i].write(array[i+m])
        b -= 1
        aux = b-m
        left = m-1
        right = b
        while right > left and left >= a:
            if this.aux[aux] >= array[left]:
                array[right].write(this.aux[aux])
                aux -= 1
            else:
                array[right].write(array[left])
                left -= 1
            right -= 1
        while right > left:
            array[right].write(this.aux[aux])
            right -= 1
            aux -= 1

    def mergeInPlace(this, array, a, m, b, check):
        if checkMergeBounds(array, a, m, b, this.rotate):
            return
        if check:
            b = lrBinarySearch(array, m, b, array[m-1].read(), True)
            a = lrBinarySearch(array, a, m-1, array[m].read(), False)
        i: int
        j: int
        k: int
        if m-a <= b-m:
            i = a
            j = m
            while i < j and j < b:
                if array[i] > array[j]:
                    k = lrBinarySearch(array, j, b, array[i].read(), True)
                    this.rotate(array, i, j, k)
                    i += k-j
                    j = k
                else:
                    i += 1
        else:
            i = m-1
            j = b-1
            while j > i and i >= a:
                if array[i] > array[j]:
                    k = lrBinarySearch(array, a, i, array[j].read(), False)
                    i += 1
                    this.rotate(array, k, i, j+1)
                    j -= i-k
                    i = k-1
                else:
                    j -= 1

    def __rotateMerge(this, array, a, m, m1, m2, m3, b):
        this.rotate(array, m1, m, m2)
        if m1-a > 0 and m3-m1 > 0:
            this.merge(array, a, m1, m3, False)
        m3 += 1
        if m2-m3 > 0 and b-m2 > 0:
            this.merge(array, m3, m2, b, False)

    def merge(this, array, a, m, b, check):
        if checkMergeBounds(array, a, m, b, this.rotate):
            return
        if check:
            b = lrBinarySearch(array, m, b, array[m-1].read(), True)
            a = lrBinarySearch(array, a, m-1, array[m].read(), False)
        size: int
        m1: int
        m2: int
        m3: int
        if b-m < m-a:
            size = b-m
            if size <= 8:
                this.mergeInPlace(array, a, m, b, False)
            elif size <= this.__capacity:
                this.__mergeDown(array, a, m, b)
            else:
                m2 = m+size//2
                m1 = lrBinarySearch(array, a, m, array[m2].read(), False)
                m3 = m2-(m-m1)
                m2 += 1
                this.__rotateMerge(array, a, m, m1, m2, m3, b)
        else:
            size = m-a
            if size <= 8:
                this.mergeInPlace(array, a, m, b, False)
            elif size <= this.__capacity:
                this.__mergeUp(array, a, m, b)
            else:
                m1 = a+(m-a)//2
                m2 = lrBinarySearch(array, m, b, array[m1].read(), True)
                m3 = m1+(m2-m)
                this.__rotateMerge(array, a, m, m1, m2, m3, b)


class UtilsIterablesSort:
    def __init__(this, size, aux):
        this.__merge = UtilsIterablesSortMerge(size, aux)

    def getReversedRuns(this, array, a, b):
        i = a
        while i < b-1:
            if array[i] <= array[i+1]:
                break
            i += 1
        if i-a > 8:
            reverse(array, a, i+1)
        return i == b

    def sort(this, array, a, b):
        if this.getReversedRuns(array, a, b):
            return
        if b-a > 32:
            m = a+((b-a)//2)
            this.sort(array, a, m)
            this.sort(array, m, b)
            this.__merge.merge(array, a, m, b, True)
        else:
            binaryInsertionSort(array, a, b)


@Sort("Merge Sorts", "Adaptive Rotate MergeSort [Utils.Iterables.sort]", "Adaptive Rotate Merge")
def UtilsIterablesSortRun(array):
    mode: int
    mode = sortingVisualizer.getUserInput(
        "Insert buffer size (default = "+str(len(array)//8)+")", str(len(array)//8), parseInt)
    UtilsIterablesSort(mode, None).sort(array, 0, len(array))
