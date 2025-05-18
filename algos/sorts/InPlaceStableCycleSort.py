class InPlaceStableCycleSort:
    def __init__(this, rot):
        if rot is None:
            this.rotate = UniV_getUserRotation("Select rotation algorithm (default: Cycle Reverse)").indexed
        else:
            this.rotate = sortingVisualizer.getRotationByName(rot).indexed

    def getRank(this, array, a, b, r):
        c = 0
        ce = 0
        i = a
        while i < b:
            if i == r:
                i += 1
                continue
            
            cmp = compareValues(
                UniV_invisibleRead(array, i), 
                UniV_invisibleRead(array, r)
            )
            c += int(cmp == -1)
            ce += int(cmp <= 0)
            i += 1
        return c, ce

    def selectMedian(this, array, a, b):
        med = (b-a)//2
        min_: int
        max_: int
        min_, max_ = findMinMaxIndices(array, a, b)
        r: int
        re: int
        r, re = this.getRank(array, a, b, min_)
        if med >= r and med <= re:
            return array[min_].copy()
        r, re = this.getRank(array, a, b, max_)
        if med >= r and med <= re:
            return array[max_].copy()
        i = a
        while True:
            if array[i] > array[min_] and array[i] < array[max_]:
                r, re = this.getRank(array, a, b, i)
                if med >= r and med <= re:
                    return array[i].copy()
                elif re < med:
                    min_ = i
                else:
                    max_ = i
            i += 1

    def resetBits(this, array, pa, pb, bLen):
        for _ in range(abs(int(bLen))):
            compSwap(array, pa, pb)
            pa += 1
            pb += 1

    def initBitBuffer(this, array, a, b, piv, bLen):
        rotate = this.rotate
        p = b
        aCnt = 0
        bCnt = 0
        tCnt = 0
        i = b
        while i > a and tCnt < 2*bLen:
            pCmp = compareValues(
                UniV_invisibleRead(array, i - 1), 
                piv
            )
            if aCnt < bLen and pCmp < 0:
                rotate(array, i, p-tCnt, p)
                p = i+tCnt
                tCnt += 1
                aCnt += 1
            elif bCnt < bLen and pCmp > 0:
                rotate(array, i, p-tCnt, p)
                p = i+tCnt
                tCnt += 1
                rotate(array, i-1, i, p-bCnt)
                bCnt += 1
            i -= 1
        rotate(array, p-tCnt, p, b)
        if tCnt == 2*bLen:
            return False
        b1 = b-tCnt
        if aCnt < bLen and bCnt < bLen:
            binaryInsertionSort(array, b1, b)
            rotate(array, a, b1, b-bCnt)
            return True
        eCnt = 0
        eLen = tCnt-bLen
        p = b1
        i = b1
        while eCnt < eLen:
            if array[i-1] == piv:
                rotate(array, i, p-eCnt, p)
                p = i+eCnt
                eCnt += 2
            i -= 1
        rotate(array, p-eLen, p, b1)
        rotate(array, b-2*bLen, b1, b-bCnt)
        return False

    def blockCyclePartitionDest(this, array, a, a1, b1, b, pa, pb, piv, bLen, cmp):
        d = a1
        e = 0
       
        pCmp = compareValues(UniV_invisibleRead(array, a1), piv)
        i = a1+bLen
        while i < b:
            vCmp = compareValues(UniV_invisibleRead(array, i), piv)
            if vCmp < pCmp:
                d += bLen
            elif (i < b1 and compareValues(
                UniV_invisibleRead(array, pa+(i-a)//bLen),
                UniV_invisibleRead(array, pb+(i-a)//bLen)
            ) != cmp and vCmp == pCmp):
                e += 1
            i += bLen
        while True:
            if compareValues(
                UniV_invisibleRead(array, pa+(d-a)//bLen),
                UniV_invisibleRead(array, pb+(d-a)//bLen)
            ) != cmp:
                if e <= 0:
                    break
                e -= 1
            d += bLen
        return d

    def blockCyclePartition(this, array, a, b, pa, pb, piv, bLen, cmp):
        i = a
        while i < b:
            if compareValues(
                UniV_invisibleRead(array, pa+(i-a)//bLen),
                UniV_invisibleRead(array, pb+(i-a)//bLen)
            ) != cmp:
                j = i
                while True:
                    k = this.blockCyclePartitionDest(
                        array, a, i, j, b, pa, pb, piv, bLen, cmp)
                    array[pa+(k-a)//bLen].swap(array[pb+(k-a)//bLen])
                    if k == i:
                        break
                    blockSwap(array, i, k, bLen)
                    j = k
            i += bLen

    def merge(this, array, cnt, a, m, b, piv):
        rotate = this.rotate
        m1 = lrBinarySearch(array, m, b, piv, True)
        m2 = lrBinarySearch(array, m1, b, piv, False)
        aCnt = m1-m
        mCnt = m2-m1
        bCnt = b-m2
        rotate(array, a+cnt[0], m, m1)
        cnt[0] += aCnt
        rotate(array, a+cnt[0]+cnt[1], m1, m2)
        cnt[1] += mCnt
        cnt[2] += bCnt

    def mergeEasy(this, array, a, m, b, piv):
        rotate = this.rotate
        b = lrBinarySearch(array, m, b, piv, False)
        m1 = lrBinarySearch(array, a, m, piv, True)
        m2 = lrBinarySearch(array, m1, m, piv, False)
        rotate(array, m2, m, b)
        b = lrBinarySearch(array, m2, b-(m-m2), piv, True)
        rotate(array, m1, m2, b)

    def partition(this, array, a, b, piv):
        rotate = this.rotate
        n = b-a
        bLen = int(math.sqrt(n-1))+1
        if this.initBitBuffer(array, a, b, piv, bLen):
            return True
        b1 = b-2*bLen
        pa = b1
        pb = b1+bLen
        cmp = 1
        i = a
        while i < b1:
            this.blockCyclePartition(array, i, min(
                i+bLen, b1), pa, pb, piv, 1, cmp)
            i += bLen
            cmp = -cmp
        this.resetBits(array, pa, pb, bLen)
        p = a
        cnt = [0, 0, 0]
        i = a
        while i < b1:
            this.merge(array, cnt, p, i, min(i+bLen, b1), piv)
            while cnt[0] >= bLen:
                cnt[0] -= bLen
                p += bLen
            while cnt[1] >= bLen:
                rotate(array, p, p+cnt[0], p+cnt[0]+bLen)
                cnt[1] -= bLen
                p += bLen
            while cnt[2] >= bLen:
                rotate(array, p, p+cnt[0]+cnt[1], p+cnt[0]+cnt[1]+bLen)
                cnt[2] -= bLen
                p += bLen
            i += bLen
        this.blockCyclePartition(array, a, p, pa, pb, piv, bLen, 1)
        this.resetBits(array, pa, pb, bLen)
        this.mergeEasy(array, p, b1, b, piv)
        this.mergeEasy(array, a, p, b, piv)
        return False

    def stableCycleDest(this, array, a, a1, b1, b, p, piv, cmp):
        d = a1
        e = 0
        i = a1+1
        while i < b:
            pCmp = compareValues(UniV_invisibleRead(array, i), piv)
            bit = pCmp == cmp or pCmp == 0
            val = UniV_invisibleRead(array, p+i-a) if bit else UniV_invisibleRead(array, i)
            vCmp = compareValues(val, UniV_invisibleRead(array, a1))
            if vCmp == -1:
                d += 1
            elif i < b1 and (not bit) and vCmp == 0:
                e += 1
            i += 1
        while True:
            pCmp = compareValues(UniV_invisibleRead(array, d), piv)
            bit = pCmp == cmp or pCmp == 0
            if not bit:
                if e == 0:
                    break
                e -= 1
            d += 1
        return d

    def stableCycle(this, array, a, b, p, piv, cmp):
        i = a
        while i < b:
            pCmp = compareValues(UniV_invisibleRead(array, i), piv)
            bit = pCmp == cmp or pCmp == 0
            if not bit:
                j = i
                while True:
                    k = this.stableCycleDest(array, a, i, j, b, p, piv, cmp)
                    if k == i:
                        break
                    t = array[i].copy()
                    array[i].write(array[k])
                    array[k].write(array[p+k-a])
                    array[p+k-a].write(t)
                    j = k
                array[i].swap(array[p+i-a])
            i += 1

    def sort(this, array, a, b):
        length = b-a
        if length <= 32:
            binaryInsertionSort(array, a, b)
            return
        piv = this.selectMedian(array, a, b)
        if this.partition(array, a, b, piv):
            return
        m2 = lrBinarySearch(array, a, b, piv, False)
        m1 = lrBinarySearch(array, a, m2, piv, True)
        h1 = m1-a
        h2 = b-m2
        hMax = max(h1, h2)
        a1 = a+hMax
        b1 = b-hMax
        this.stableCycle(array, a, a+h1, b1, piv, 1)
        blockSwap(array, a+h1, b1+h1, hMax-h1)
        blockSwap(array, a, b1, hMax-h2)
        this.stableCycle(array, a1-h2, a1, b-h2, piv, -1)


@Sort("Hybrid Sorts", "In-Place Stable Cycle Sort", "In-Place Stable Cycle")
def inPlaceStableCycleRun(array):
    InPlaceStableCycleSort(None).sort(array, 0, len(array))
