class PacheSort:
    MIN_INSERT = 32
    MIN_HEAP = 255

    def log2(this, n):
        return int(math.log2(n))

    def siftDown(this, array, pos, len, root, t):
        curr = root
        cmp = 1 if (javaNumberOfLeadingZeros(root+1) & 1) == 1 else -1
        left = 2*curr+1
        while left < len:
            next = left
            gChild = 2*left+1
            for node in [left+1, gChild+0, gChild+1, gChild+2, gChild+3]:
                if node >= len:
                    break
                if compareValues(array[pos+node], array[pos+next]) == cmp:
                    next = node
            if next >= gChild:
                if compareValues(array[pos+next], t) == cmp:
                    array[pos+curr].write(array[pos+next])
                    curr = next
                    left = 2*curr+1
                    parent = (next-1)//2
                    if compareValues(array[pos+parent], t) == cmp:
                        array[pos+curr].write(t)
                        t = array[pos+parent].copy()
                        array[pos+parent].write(array[pos+curr])
                else:
                    break
            else:
                if compareValues(array[pos+next], t) == cmp:
                    array[pos+curr].write(array[pos+next])
                    curr = next
                break
        array[pos+curr].write(t)

    def heapify(this, array, pos, len):
        i = (len-1)//2
        while i >= 0:
            this.siftDown(array, pos, len, i, array[pos+i].copy())
            i -= 1

    def minMaxHeap(this, array, a, b):
        pos = a
        len = b-a
        this.heapify(array, pos, len)
        i = len
        while i > 1:
            i -= 1
            t = array[pos+i].copy()
            array[pos+i].write(array[pos])
            this.siftDown(array, pos, i, 0, t)

    def selectMinMax(this, array, a, b, s):
        this.heapify(array, a, b-a)
        i = 0
        while i < s:
            b -= 1
            t = array[b].copy()
            array[b].write(array[a])
            this.siftDown(array, a, b-a, 0, t)
            i += 1
        i = 0
        while i < s:
            b -= 1
            t = array[b].copy()
            c = 1
            if array[a+c+1] < array[a+c]:
                c += 1
            array[b].write(array[a+c])
            this.siftDown(array, a, b-a, c, t)
            i += 1
        a1 = a+s
        while a1 > a:
            a1 -= 1
            array[a1].swap(array[b])
            b += 1

    def optiLazyHeap(this, array, a, b, s):
        j = a
        while j < b:
            max = j
            i = max+1
            while i < min(j+s, b):
                if array[i] > array[max]:
                    max = i
                i += 1
            array[j].swap(array[max])
            j += s
        j = b
        while j > a:
            k = a
            i = k+s
            while i < j:
                if array[i] > array[k]:
                    k = i
                i += s
            j -= 1
            k1 = j
            i = k+1
            while i < min(k+s, j):
                if array[i] > array[k1]:
                    k1 = i
                i += 1
            if k1 == j:
                array[k].swap(array[j])
            else:
                t = array[j].read()
                array[j].write(array[k])
                array[k].write(array[k1])
                array[k1].write(t)

    def sortBucket(this, array, a, b, s, val):
        i = b-1
        while i >= a:
            if array[i] == val:
                b -= 1
                array[i].swap(array[b])
            i -= 1
        this.optiLazyHeap(array, a, b, s)

    def sort(this, array, a, b):
        if b-a <= this.MIN_HEAP:
            this.minMaxHeap(array, a, b)
            return
        log = this.log2(b-a-1)+1
        pCnt = (b-a)//(log**2)
        bitLen = (pCnt+1)*log
        a1 = a+bitLen
        b1 = b-bitLen
        this.selectMinMax(array, a, b, bitLen)
        if array[a1] < array[b1-1]:
            a2 = a1
            i = 0
            while i < pCnt:
                array[a2].swap(array[random.randrange(a2, b1)])
                i += 1
                a2 += 1
            this.minMaxHeap(array, a1, a2)
            cnts = BitArray(array, a, b1, pCnt+1, log)
            i = a2
            while i < b1:
                cnts.incr(lrBinarySearch(array, a1, a2, array[i], True)-a1)
                i += 1
            i = 1
            sum = cnts.get(0)
            while i < pCnt+1:
                sum += cnts.get(i)
                cnts.set(i, sum)
                i += 1
            i = 0
            j = 0
            while i < pCnt:
                cur = cnts.get(i)
                loc = lrBinarySearch(array, a1+i, a2, array[a2+j], True)-a1
                while j < cur:
                    if loc == i:
                        j += 1
                        loc = lrBinarySearch(array, a1+i, a2, array[a2+j], True)-a1
                    else:
                        cnts.decr(loc)
                        dest = cnts.get(loc)
                        while True:
                            newLoc = lrBinarySearch(
                                array, a1+i, a2, array[a2+dest], True)-a1
                            if newLoc != loc:
                                loc = newLoc
                                break
                            cnts.decr(loc)
                            dest -= 1
                        array[a2+j].swap(array[a2+dest])
                j = lrBinarySearch(array, a2+j, b1, array[a1+i], False)-a2
                i += 1
            cnts.free()
            j = a2
            i = 0
            while i < pCnt:
                j1 = lrBinarySearch(array, j, b1, array[a1+i], False)
                this.sortBucket(array, j, j1, log, array[a1+1].read())
                j = j1
                i += 1
            this.optiLazyHeap(array, j, b1, log)
            HeliumSort__mergeWithBufferFW(None, array, a1, a2, b1, a, True)
            this.minMaxHeap(array, a, a+pCnt)


@Sort("Hybrid Sorts", "Pache Sort", "Pache Sort")
def pacheSortRun(array):
    PacheSort().sort(array, 0, len(array))
