class AdvancedLogMergeSort:
    MIN_INSERT = 16

    def __init__(this, rot):
        if rot is None:
            this.rotate = UniV_getUserRotation("Select rotation algorithm", "Cycle Reverse").indexed
        else:
            this.rotate = sortingVisualizer.getRotationByName(rot).indexed
        this.aux = None

    def log2(this, n):
        return 31-javaNumberOfLeadingZeros(n)

    def calcBLen(this, n):
        h = n//2
        r = min(64, h//2)
        c = h//r
        while c*(this.log2(c-1)+1)+c <= h//2:
            r -= 1
            c = h//r
        return r+1

    def pivCmp(this, v, piv, pCmp):
        return compareValues(v, piv) < pCmp

    def pivBufXor(this, array, pa, pb, v, wLen):
        while wLen > 0:
            wLen -= 1
            if (v & 1) == 1:
                array[pa+wLen].swap(array[pb+wLen])
            v >>= 1

    def pivBufGet(this, array, pa, piv, pCmp, wLen, bit):
        r = 0
        while wLen > 0:
            r <<= 1
            r |= int(not this.pivCmp(array[pa], piv, pCmp)) ^ bit
            wLen -= 1
            pa += 1
        return r

    def blockCycle(this, array, p, n, p1, bLen, wLen, piv, pCmp, bit):
        for i in range(n):
            dest = this.pivBufGet(array, p+i*bLen, piv, pCmp, wLen, bit)
            while dest != i:
                blockSwap(array, p+i*bLen, p+dest*bLen, bLen)
                dest = this.pivBufGet(array, p+i*bLen, piv, pCmp, wLen, bit)
            this.pivBufXor(array, p+i*bLen, p1+i*bLen, i, wLen)

    def mergeFWExt(this, array, a, m, b):
        s = m-a
        bidirArrayCopy(array, a, this.aux, 0, s)
        i = 0
        j = m
        while i < s and j < b:
            if this.aux[i] <= array[j]:
                array[a].write(this.aux[i])
                i += 1
            else:
                array[a].write(array[j])
                j += 1
            a += 1
        while i < s:
            array[a].write(this.aux[i])
            a += 1
            i += 1

    def mergeBWExt(this, array, a, m, b):
        s = b-m
        bidirArrayCopy(array, m, this.aux, 0, s)
        i = s-1
        j = m-1
        while i >= 0 and j >= a:
            b -= 1
            if this.aux[i] >= array[j]:
                array[b].write(this.aux[i])
                i -= 1
            else:
                array[b].write(array[j])
                j -= 1
        while i >= 0:
            b -= 1
            array[b].write(this.aux[i])
            i -= 1

    def blockMergeHelper(this, array, a, m, b, p, bLen, piv, pCmp, bit):
        if m-a <= bLen:
            this.mergeFWExt(array, a, m, b)
            return
        bidirArrayCopy(array, m-bLen, this.aux, 0, bLen)
        wLen = this.log2((b-a)//bLen-2)+1
        bCnt = 0
        i = a
        j = m
        k = 0
        pc = p
        while i < m-bLen and j+bLen-1 < b:
            if array[i+bLen-1] <= array[j+bLen-1]:
                this.pivBufXor(array, i, pc, k, wLen)
                i += bLen
            else:
                this.pivBufXor(array, j, pc, (k << 1) | 1, wLen+1)
                j += bLen
            pc += bLen
            bCnt += 1
            k += 1
        while i < m-bLen:
            this.pivBufXor(array, i, pc, k, wLen)
            i += bLen
            pc += bLen
            bCnt += 1
            k += 1
        bidirArrayCopy(array, a, array, m-bLen, bLen)
        a1 = a+bLen
        this.blockCycle(array, a1, bCnt, p, bLen, wLen, piv, pCmp, bit)
        f = a1
        left = this.pivCmp(array[a1+wLen], piv, pCmp) ^ bool(bit)
        if not left:
            array[a1+wLen].swap(array[p+wLen])
        k = 1
        j = a
        while k < bCnt:
            nxt = a1+k*bLen
            frag = this.pivCmp(array[nxt+wLen], piv, pCmp) ^ bool(bit)
            if not frag:
                array[nxt+wLen].swap(array[p+(nxt+wLen-a1)])
            if left ^ frag:
                i = f
                f = nxt
                while i < nxt:
                    cmp = compareValues(array[i], array[f])
                    if cmp < 0 or (left and cmp == 0):
                        array[j].write(array[i])
                        i += 1
                    else:
                        array[j].write(array[f])
                        f += 1
                    j += 1
                left = not left
            k += 1
        if left:
            k = a1+bCnt*bLen
            i = f
            f = k
            while i < k and f < b:
                if array[i] <= array[f]:
                    array[j].write(array[i])
                    i += 1
                else:
                    array[j].write(array[f])
                    f += 1
                j += 1
            if f == b:
                while i < k:
                    array[j].write(array[i])
                    j += 1
                    i += 1
                bidirArrayCopy(this.aux, 0, array, b-bLen, bLen)
                return
        i = 0
        while i < bLen and f < b:
            if this.aux[i] <= array[f]:
                array[j].write(this.aux[i])
                i += 1
            else:
                array[j].write(array[f])
                f += 1
            j += 1
        while i < bLen:
            array[j].write(this.aux[i])
            j += 1
            i += 1

    def blockMergeEasy(this, array, a, m, b, p, bLen, piv, pCmp, bit):
        if b-m <= bLen:
            this.mergeBWExt(array, a, m, b)
            return
        if m-a <= bLen:
            this.mergeFWExt(array, a, m, b)
            return
        a1 = a+(m-a) % bLen
        this.blockMergeHelper(array, a1, m, b, p, bLen, piv, pCmp, bit)
        this.mergeFWExt(array, a, a1, b)

    def blockMerge(this, array, a, m, b, bLen):
        rotate = this.rotate
        l = m-a
        r = b-m
        lCnt = (l+r+1)//2
        med: Value
        if r < l:
            if r <= bLen:
                this.mergeBWExt(array, a, m, b)
                return False
            la = 0
            lb = r
            while la < lb:
                lm = (la+lb)//2
                if array[m+lm] <= array[a+(lCnt-lm)-1]:
                    la = lm+1
                else:
                    lb = lm
            if la == 0:
                med = array[a+lCnt-1].copy()
            elif array[m+la-1] > array[a+(lCnt-la)-1]:
                med = array[m+la-1].copy()
            else:
                med = array[a+(lCnt-la)-1].copy()
        else:
            if l <= bLen:
                this.mergeFWExt(array, a, m, b)
                return False
            la = 0
            lb = l
            while la < lb:
                lm = (la+lb)//2
                if array[a+lm] < array[m+(lCnt-lm)-1]:
                    la = lm+1
                else:
                    lb = lm
            if l == r and la == l:
                med = array[m-1].copy()
            elif la == 0:
                med = array[m+lCnt-1].copy()
            elif array[a+la-1] >= array[m+(lCnt-la)-1]:
                med = array[a+la-1].copy()
            else:
                med = array[m+(lCnt-la)-1].copy()
        m1 = lrBinarySearch(array, a, m, med, True)
        m2 = lrBinarySearch(array, m, b, med, False)
        ms2 = m-lrBinarySearch(array, m1, m, med, False)
        ms1 = lrBinarySearch(array, m, m2, med, True)-m
        rotate(array, m-ms2, m, m2)
        rotate(array, m1, m-ms2, m+ms1-ms2)
        this.blockMergeEasy(array, a, m1, m1+ms1, a+lCnt, bLen, med, 0, 0)
        this.blockMergeEasy(array, m2-ms2, m2, b, a, bLen, med, 1, 1)
        return m2-m1-(ms2+ms1) <= lCnt

    def blockMergeWithBufHelper(this, array, a, m, b, pa, pb, bLen):
        if m-a <= bLen:
            this.mergeFWExt(array, a, m, b)
            return
        bidirArrayCopy(array, m-bLen, this.aux, 0, bLen)
        bCnt = 0
        maxBCnt = (b-a)//bLen-1
        wLen = this.log2(maxBCnt)+1
        pos = BitArray(array, pa, pb, maxBCnt, wLen)
        bits = BitArray(array, pb-maxBCnt, pb+pb-pa-maxBCnt, maxBCnt, 1)
        a1 = a+bLen
        i = a
        j = m
        k: int
        posV: int
        while i < m-bLen and j+bLen-1 < b:
            if array[i+bLen-1] <= array[j+bLen-1]:
                if i == a:
                    posV = (m-a1)//bLen-1
                else:
                    posV = (i-a1)//bLen
                bits.setXor(bCnt, 1)
                i += bLen
            else:
                posV = (j-a1)//bLen
                j += bLen
            if bCnt != posV:
                pos.setXor(bCnt, posV+1)
            bCnt += 1
        while i < m-bLen:
            if i == a:
                posV = (m-a1)//bLen-1
            else:
                posV = (i-a1)//bLen
            if bCnt != posV:
                pos.setXor(bCnt, posV+1)
            bits.setXor(bCnt, 1)
            i += bLen
            bCnt += 1
        bidirArrayCopy(array, a, array, m-bLen, bLen)
        for i in range(bCnt):
            k = pos.get(i)
            if k > 0:
                bidirArrayCopy(array, a1+i*bLen, array, a, bLen)
                j = i
                while True:
                    bidirArrayCopy(array, a1+(k-1)*bLen,
                                   array, a1+j*bLen, bLen)
                    pos.setXor(j, k)
                    j = k-1
                    k = pos.get(j)
                    if not (k != i+1):
                        break
                bidirArrayCopy(array, a, array, a1+j*bLen, bLen)
                pos.setXor(j, k)
        f = a1
        left = bits.get(0) != 0
        if left:
            bits.setXor(0, 1)
        k = 1
        j = a
        while k < bCnt:
            nxt = a1+k*bLen
            frag = bits.get(k) != 0
            if frag:
                bits.setXor(k, 1)
            if left ^ frag:
                i = f
                f = nxt
                while i < nxt:
                    cmp = compareValues(array[i], array[f])
                    if cmp < 0 or (left and cmp == 0):
                        array[j].write(array[i])
                        i += 1
                    else:
                        array[j].write(array[f])
                        f += 1
                    j += 1
                left = not left
            k += 1
        if left:
            k = a1+bCnt*bLen
            i = f
            f = k
            while i < k and f < b:
                if array[i] <= array[f]:
                    array[j].write(array[i])
                    i += 1
                else:
                    array[j].write(array[f])
                    f += 1
                j += 1
            if f == b:
                while i < k:
                    array[j].write(array[i])
                    j += 1
                    i += 1
                bidirArrayCopy(this.aux, 0, array, b-bLen, bLen)
                return
        i = 0
        while i < bLen and f < b:
            if this.aux[i] <= array[f]:
                array[j].write(this.aux[i])
                i += 1
            else:
                array[j].write(array[f])
                f += 1
            j += 1
        while i < bLen:
            array[j].write(this.aux[i])
            j += 1
            i += 1

    def blockMergeWithBuf(this, array, a, m, b, pa, pb, bLen):
        if b-m <= bLen:
            this.mergeBWExt(array, a, m, b)
            return
        if m-a <= bLen:
            this.mergeFWExt(array, a, m, b)
            return
        a1 = a+(m-a) % bLen
        this.blockMergeWithBufHelper(array, a1, m, b, pa, pb, bLen)
        this.mergeFWExt(array, a, a1, b)

    def pureLogMergeSort(this, array, a, b, bLen):
        j = b-a
        while (j+1)//2 >= this.MIN_INSERT:
            j = (j+1)//2
        speed = sortingVisualizer.getSpeed()
        sortingVisualizer.setSpeed(max(int(10*(len(array)/2048)), speed*2))
        i = a
        while i < b:
            binaryInsertionSort(array, i, min(b, i+j))
            i += j
        sortingVisualizer.setSpeed(speed)
        while j < b-a:
            k = a
            while k+j < b and not this.blockMerge(array, k, k+j, min(b, k+2*j), bLen):
                k += 2*j
            i = k+2*j
            while i+j < b:
                this.blockMergeWithBuf(
                    array, i, i+j, min(b, i+2*j), k, k+j, bLen)
                i += 2*j
            j *= 2

    def sort(this, array, a, b, mem):
        length = b-a
        if length <= this.MIN_INSERT:
            binaryInsertionSort(array, a, b)
            return
        bLen = max(this.calcBLen(length), min(mem, length))
        this.aux = sortingVisualizer.createValueArray(bLen)
        this.pureLogMergeSort(array, a, b, bLen)


@Sort("Block Merge Sorts", "Advanced Log Merge Sort", "Advanced Log Merge")
def advancedLogMergeSortRun(array):
    advancedLogMerge = AdvancedLogMergeSort(None)
    mem = sortingVisualizer.getUserInput("Set block size (default: calculates minimum block length for current length)", str(
        advancedLogMerge.calcBLen(len(array))), parseInt)
    advancedLogMerge.sort(array, 0, len(array), mem)
