class LogSort:
    def productLog(this, n):
        r = 1
        while (r << r)+r-1 < n:
            r += 1
        return r

    def insertionSort(this, array, a, n):
        insertionSort(array, a, a+n)

    def quickSelect(this, array, a, n, p):
        while n > 16:
            a1 = a+n/2
            a2 = a+n-1
            if array[a1] > array[a]:
                array[a1].swap(array[a])
            if array[a] > array[a2]:
                array[a].swap(array[a2])
            if array[a1] > array[a]:
                array[a1].swap(array[a])
            i = a
            j = a+n
            while True:
                i += 1
                while i < j and array[i] < array[a]:
                    i += 1
                j -= 1
                while j >= i and array[j] > array[a]:
                    j -= 1
                if i < j:
                    array[i].swap(array[j])
                else:
                    array[a].swap(array[j])
                    break
            m = j-a
            if p < m:
                n = m
            elif p > m:
                n -= m+1
                p -= m+1
                a = j+1
            else:
                return
        this.insertionSort(array, a, n)

    def medianOf9(this, array, swap, a, n):
        s = (n-1)//8
        i = 0
        j = a
        while i < 9:
            swap[i].write(array[j])
            i += 1
            j += s
        insertionSort(swap, 0, 9)
        return swap[4].copy()

    def smartMedian(this, array, swap, a, n, bLen):
        cbrt = 32
        while cbrt*cbrt*cbrt < n and cbrt < 1024:
            cbrt *= 2
        d = min(bLen, cbrt)
        d -= d % 2
        s = n//d
        i = 0
        j = a+int(random.random()*s)
        while i < d:
            swap[i].write(array[j])
            i += 1
            j += s
        this.quickSelect(swap, 0, d, d//2)
        return swap[d//2].copy()

    def blockRead(this, array, a, piv, wLen, pCmp):
        r = 0
        i = 0
        while wLen > 0:
            wLen -= 1
            r |= int(compareValues(array[a], piv) < pCmp) << i
            a += 1
            i += 1
        return r

    def blockXor(this, array, a, b, v):
        while v > 0:
            if (v & 1) > 0:
                array[a].swap(array[b])
            v >>= 1
            a += 1
            b += 1

    def partitionEasy(this, array, swap, a, n, piv, pCmp):
        p = a
        ps = 0
        i = n
        while i > 0:
            if compareValues(array[p], piv) < pCmp:
                array[a].write(array[p])
                a += 1
            else:
                swap[ps].write(array[p])
                ps += 1
            i -= 1
            p += 1
        bidirArrayCopy(swap, 0, array, a, ps)
        return a

    def partition(this, array, swap, a, n, bLen, piv, pCmp):
        if n <= bLen:
            return this.partitionEasy(array, swap, a, n, piv, pCmp)
        p = a
        l = 0
        r = 0
        lb = 0
        rb = 0
        i = 0
        while i < n:
            if compareValues(array[a+i], piv) < pCmp:
                array[p+l].write(array[a+i])
                l += 1
            else:
                swap[r].write(array[a+i])
                r += 1
            if l == bLen:
                p += bLen
                l = 0
                lb += 1
            if r == bLen:
                bidirArrayCopy(array, p, array, p+bLen, l)
                bidirArrayCopy(swap, 0, array, p, bLen)
                p += bLen
                r = 0
                rb += 1
            i += 1
        bidirArrayCopy(swap, 0, array, p+l, r)
        x = lb < rb
        min_ = lb if x else rb
        m = a+lb*bLen
        if min_ > 0:
            max_ = lb+rb-min_
            wLen = log2(min_)
            j = a
            k = a
            v = 0
            i = min_
            while i > 0:
                while not (compareValues(array[j+wLen], piv) < pCmp):
                    j += bLen
                while compareValues(array[k+wLen], piv) < pCmp:
                    k += bLen
                this.blockXor(array, j, k, v)
                i -= 1
                v += 1
                j += bLen
                k += bLen
            j = p-bLen if x else a
            k = j
            s = (-bLen)if x else bLen
            i = max_
            while i > 0:
                if x ^ (compareValues(array[k+wLen], piv) < pCmp):
                    blockSwap(array, j, k, bLen)
                    j += s
                    i -= 1
                k += s
            j = 0
            ps = a if x else m
            pa = ps
            pb = m if x else a
            mask = (int(x) << wLen)-int(x)
            i = min_
            while i > 0:
                k = mask ^ this.blockRead(array, ps, piv, wLen, pCmp)
                while j != k:
                    blockSwap(array, ps, pa+k*bLen, bLen)
                    k = mask ^ this.blockRead(array, ps, piv, wLen, pCmp)
                this.blockXor(array, ps, pb, j)
                j += 1
                ps += bLen
                pb += bLen
                i -= 1
        if l > 0:
            bidirArrayCopy(array, p, swap, 0, l)
            bidirArrayCopy(array, m, array, m+l, rb*bLen)
            bidirArrayCopy(swap, 0, array, m, l)
        return m+l

    def logSort(this, array, swap, a, n, bLen):
        while n > 24:
            piv = this.medianOf9(array, swap, a, n)if n < 2048 else this.smartMedian(
                array, swap, a, n, bLen)
            p = this.partition(array, swap, a, n, bLen, piv, 1)
            m = p-a
            if m == n:
                p = this.partition(array, swap, a, n, bLen, piv, 0)
                n = p-a
                continue
            this.logSort(array, swap, p, n-m, bLen)
            n = m
        this.insertionSort(array, a, n)

    def sort(this, array, a, n, bLen):
        bLen = max(9, min(n, bLen))
        swap = sortingVisualizer.createValueArray(bLen)
        this.logSort(array, swap, a, n, bLen)


@Sort("Quick Sorts", "Log Sort", "Log Sort")
def logSortRun(array):
    logsort = LogSort()
    bLen = sortingVisualizer.getUserInput(
        "Set block size (default: calculates minimum block length for current length)", str(logsort.productLog(len(array))), parseInt)
    logsort.sort(array, 0, len(array), bLen)
