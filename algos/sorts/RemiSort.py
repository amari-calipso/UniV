class RemiSort:
    def __init__(this):
        this.keys = None
        this.buf = None
        this.heap = None
        this.p = None
        this.pa = None

    def ceilCbrt(this, n):
        a = 0
        b = min(1291, n)
        while a < b:
            m = (a+b)//2
            if m**3 >= n:
                b = m
            else:
                a = m+1
        return a

    def siftDown(this, array, r, len_, a, t):
        j = r
        while 2*j+1 < len_:
            j = 2*j+1
            if j+1 < len_:
                cmp = compareValues(
                    array[a+this.keys[j+1].readInt()], array[a+this.keys[j].readInt()])
                if cmp > 0 or (cmp == 0 and this.keys[j+1] > this.keys[j]):
                    j += 1
        cmp = compareValues(array[a+t.readInt()],
                            array[a+this.keys[j].readInt()])
        while cmp > 0 or (cmp == 0 and this.keys[j] < t):
            j = (j-1)//2
            cmp = compareValues(array[a+t.readInt()],
                                array[a+this.keys[j].readInt()])
        t2: Value
        while j > r:
            t2 = this.keys[j].read()
            this.keys[j].write(t)
            t = t2
            j = (j-1)//2
        this.keys[r].write(t)

    def tableSort(this, array, a, b):
        len_ = b-a
        i = (len_-1)//2
        while i >= 0:
            this.siftDown(array, i, len_, a, this.keys[i].read())
            i -= 1
        i = len_-1
        while i > 0:
            t = this.keys[i].read()
            this.keys[i].write(this.keys[0])
            this.siftDown(array, 0, i, a, t)
            i -= 1
        for i in range(len_):
            if this.keys[i] != i:
                t = array[a+i].read()
                j = i
                next = this.keys[i].readInt()
                while True:
                    array[a+j].write(array[a+next])
                    this.keys[j].write(j)
                    j = next
                    next = this.keys[next].readInt()
                    if not (next != i):
                        break
                array[a+j].write(t)
                this.keys[j].write(j)

    def blockCycle(this, array, a, bLen, bCnt):
        for i in range(bCnt):
            if this.keys[i] != i:
                bidirArrayCopy(array, a+i*bLen, this.buf, 0, bLen)
                j = i
                next = this.keys[i].readInt()
                while True:
                    bidirArrayCopy(array, a+next*bLen, array, a+j*bLen, bLen)
                    this.keys[j].write(j)
                    j = next
                    next = this.keys[next].readInt()
                    if not (next != i):
                        break
                bidirArrayCopy(this.buf, 0, array, a+j*bLen, bLen)
                this.keys[j].write(j)

    def kWayMerge(this, array, b, bLen, rLen):
        k = len(this.p)
        size = k
        a = this.pa[0].readInt()
        a1 = this.pa[1].readInt()
        for i in range(k):
            this.heap[i].write(i)
        i = (k-1)//2
        while i >= 0:
            kWayMerge.siftDown(array, this.heap, this.pa,
                          this.heap[i].readInt(), i, k)
            i -= 1
        for i in range(rLen):
            min_ = this.heap[0].readInt()
            this.buf[i].write(array[this.pa[min_].readInt()])
            this.pa[min_] += 1
            if this.pa[min_] == min(a+(min_+1)*rLen, b):
                size -= 1
                kWayMerge.siftDown(array, this.heap, this.pa,
                              this.heap[size].readInt(), 0, size)
            else:
                kWayMerge.siftDown(array, this.heap, this.pa,
                              this.heap[0].readInt(), 0, size)
        t = 0
        cnt = 0
        c = 0
        while this.pa[c]-this.p[c] < bLen:
            c += 1
        while True:
            min_ = this.heap[0].readInt()
            array[this.p[c].readInt()].write(array[this.pa[min_].readInt()])
            this.pa[min_] += 1
            this.p[c] += 1
            if this.pa[min_] == min(a+(min_+1)*rLen, b):
                size -= 1
                kWayMerge.siftDown(array, this.heap, this.pa,
                              this.heap[size].readInt(), 0, size)
            else:
                kWayMerge.siftDown(array, this.heap, this.pa,
                              this.heap[0].readInt(), 0, size)
            cnt += 1
            if cnt == bLen:
                if c > 0:
                    this.keys[t].write(this.p[c]//bLen-bLen-1)
                else:
                    this.keys[t].write(-1)
                t += 1
                c = 0
                cnt = 0
                while this.pa[c]-this.p[c] < bLen:
                    c += 1
            if not (size > 0):
                break
        while cnt > 0:
            cnt -= 1
            this.p[c] -= 1
            b -= 1
            array[b].write(array[this.p[c].readInt()])
        this.pa[k-1].write(b)
        this.keys[-1].write(-1)
        t = 0
        while this.keys[t] != -1:
            t += 1
        i = 1
        j = a
        while this.p[0] > j:
            while this.p[i] < this.pa[i]:
                this.keys[t].write(this.p[i]//bLen-bLen)
                t += 1
                while this.keys[t] != -1:
                    t += 1
                bidirArrayCopy(array, j, array, this.p[i].readInt(), bLen)
                this.p[i] += bLen
                j += bLen
            i += 1
        bidirArrayCopy(this.buf, 0, array, a, rLen)
        this.blockCycle(array, a1, bLen, (b-a1)//bLen)

    def sort(this, array, a, b):
        length = b-a
        bLen = this.ceilCbrt(length)
        rLen = bLen*bLen
        rCnt = (length-1)//rLen+1
        if rCnt < 2:
            this.keys = sortingVisualizer.createValueArray(length)
            sortingVisualizer.setNonOrigAux([this.keys])
            for i in range(length):
                this.keys[i].write(i)
            this.tableSort(array, a, b)
            return
        this.keys = sortingVisualizer.createValueArray(rLen)
        this.buf = sortingVisualizer.createValueArray(rLen)
        this.heap = sortingVisualizer.createValueArray(rCnt)
        this.p = sortingVisualizer.createValueArray(rCnt)
        this.pa = sortingVisualizer.createValueArray(rCnt)
        sortingVisualizer.setNonOrigAux([this.keys, this.heap, this.p, this.pa])
        for i in range(rLen):
            this.keys[i].write(i)
        i = a
        j = 0
        while i < b:
            this.tableSort(array, i, min(i+rLen, b))
            this.pa[j].write(i)
            i += rLen
            j += 1
        bidirArrayCopy(this.pa, 0, this.p, 0, rCnt)
        this.kWayMerge(array, b, bLen, rLen)


@Sort("Block Merge Sorts", "Remi Sort", "Remi Sort")
def remiSortRun(array):
    RemiSort().sort(array, 0, len(array))
