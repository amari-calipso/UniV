class LithiumSort:
    RUN_SIZE = 32
    SMALL_SORT = 256
    MAX_STRAT3_UNIQUE = 8
    SMALL_MERGE = 16

    def __init__(this):
        this.bufLen = 0

    def multiTriSwap(this, array, a, b, c, len):
        for i in range(len):
            t = array[a+i].copy()
            array[a+i].write(array[b+i])
            array[b+i].write(array[c+i])
            array[c+i].write(t)

    def rotate(this, array, a, m, b):
        rl = b-m
        ll = m-a
        bl = this.bufLen
        min_ = bl if rl != ll and min(bl, rl, ll) > this.SMALL_MERGE else 1
        while (rl > min_ and ll > min_) or (rl < this.SMALL_MERGE and rl > 1 and ll < this.SMALL_MERGE and ll > 1):
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
        if min == 1 or rl <= 1 or ll <= 1:
            return
        if rl < ll:
            backwardBlockSwap(array, m, this.bufPos, rl)
            i = m+rl-1
            while i >= a+rl:
                array[i].swap(array[i-rl])
                i -= 1
            backwardBlockSwap(array, this.bufPos, a, rl)
        else:
            blockSwap(array, a, this.bufPos, ll)
            i = a
            while i < b-ll:
                array[i].swap(array[i+ll])
                i += 1
            blockSwap(array, this.bufPos, b-ll, ll)

    def findKeys(this, array, a, b, q):
        n = 1
        p = b-1
        i = p
        while i > a and n < q:
            l = lrBinarySearch(array, p, p+n, array[i-1], True)-p
            if l == n or array[i-1] < array[p+l]:
                this.rotate(array, i, p, p+n)
                n += 1
                p = i-1
                insertToRight(array, i-1, p+l)
            i -= 1
        this.rotate(array, p, p+n, b)
        return n

    def sortRuns(this, array, a, b):
        speed = sortingVisualizer.getSpeed()
        sortingVisualizer.setSpeed(max(int(10*(len(array)/2048)), speed*4))
        i = a
        while i < b-this.RUN_SIZE:
            binaryInsertionSort(array, i, i+this.RUN_SIZE)
            i += this.RUN_SIZE
        if i < b:
            binaryInsertionSort(array, i, b)
        sortingVisualizer.setSpeed(speed)

    def mergeInPlaceBW(this, array, a, m, b, left):
        s = b-1
        l = m-1
        while s > l and l >= a:
            cmp = compareValues(array[l], array[s])
            if cmp > 0 if left else cmp >= 0:
                p = lrBinarySearch(array, a, l, array[s], not left)
                this.rotate(array, p, l+1, s+1)
                s -= l+1-p
                l = p-1
            else:
                s -= 1

    def mergeWithBufferBW(this, array, a, m, b, left):
        rl = b-m
        if rl <= this.SMALL_MERGE or rl > this.bufLen:
            this.mergeInPlaceBW(array, a, m, b, left)
            return
        backwardBlockSwap(array, m, this.bufPos, rl)
        l = m-1
        r = this.bufPos+rl-1
        o = b-1
        while l >= a and r >= this.bufPos:
            cmp = compareValues(array[r], array[l])
            if cmp >= 0 if left else cmp > 0:
                array[o].swap(array[r])
                r -= 1
            else:
                array[o].swap(array[l])
                l -= 1
            o -= 1
        while r >= this.bufPos:
            array[o].swap(array[r])
            o -= 1
            r -= 1

    def mergeRestWithBufferFW(this, array, a, m, b, pLen, left):
        l = this.bufPos
        r = m
        o = a
        e = l+pLen
        while l < e and r < b:
            cmp = compareValues(array[l], array[r])
            if cmp <= 0 if left else cmp < 0:
                array[o].swap(array[l])
                l += 1
            else:
                array[o].swap(array[r])
                r += 1
            o += 1
        while l < e:
            array[o].swap(array[l])
            o += 1
            l += 1

    def mergeWithScrollingBufferFW(this, array, a, m, b, p, left):
        i = a
        j = m
        while i < m and j < b:
            cmp = compareValues(array[i], array[j])
            if cmp <= 0 if left else cmp < 0:
                array[p].swap(array[i])
                i += 1
            else:
                array[p].swap(array[j])
                j += 1
            p += 1
        if i > p:
            while i < m:
                array[p].swap(array[i])
                p += 1
                i += 1
        return j

    def shift(this, array, a, m, b, left):
        if left:
            if m == b:
                return
            while m > a:
                b -= 1
                m -= 1
                array[b].swap(array[m])
        else:
            if (m == a):
                return
            while m < b:
                array[a].swap(array[m])
                a += 1
                m += 1

    def dualMergeFW(this, array, a, m, b, r):
        i = a
        j = m
        k = a-r
        while k < i and i < m:
            if array[i] <= array[j]:
                array[k].swap(array[i])
                i += 1
            else:
                array[k].swap(array[j])
                j += 1
            k += 1
        if k < i:
            this.shift(array, j-r, j, b, False)
        else:
            i2 = m-1
            j2 = b-1
            k = i2+b-j
            while i2 >= i and j2 >= j:
                if array[i2] > array[j2]:
                    array[k].swap(array[i2])
                    i2 -= 1
                else:
                    array[k].swap(array[j2])
                    j2 -= 1
                k -= 1
            while j2 >= j:
                array[k].swap(array[j2])
                k -= 1
                j2 -= 1

    def swapKeys(this, array, bits, a, b):
        if bits is None:
            array[this.keyPos+a].swap(array[this.keyPos+b])
        else:
            bits.swap(a, b)

    def compareKeys(this, array, bits, a, b):
        if bits is None:
            return compareValues(array[this.keyPos+a], array[this.keyPos+b])
        else:
            return compareValues(bits.get(a), bits.get(b))

    def blockSelect(this, array, bits, a, leftBlocks, rightBlocks, blockLen):
        total = leftBlocks+rightBlocks
        j = 0
        k = leftBlocks+1
        while j < k-1:
            min = j
            i = max(leftBlocks-1, j+1)
            while i < k:
                cmp = compareValues(
                    array[a+(i+1)*blockLen-1], array[a+(min+1)*blockLen-1])
                if cmp < 0 or (cmp == 0 and this.compareKeys(array, bits, i, min) < 0):
                    min = i
                i += 1
            if min != j:
                blockSwap(array, a+j*blockLen, a+min*blockLen, blockLen)
                this.swapKeys(array, bits, j, min)
                if k < total and min == k-1:
                    k += 1
            j += 1

    def compareMidKey(this, array, bits, i, midKey):
        if bits is None:
            return array[this.keyPos+i] < midKey
        else:
            return bits.get(i) < midKey

    def mergeBlocksWithBuf(this, array, a, midKey, leftBlocks, rightBlocks, b, blockLen, bits):
        t = leftBlocks+rightBlocks
        a1 = a+blockLen
        i = a1
        j = a
        k = -1
        l = -1
        r = leftBlocks-1
        left = True
        while l < leftBlocks and r < t:
            if left:
                while True:
                    j += blockLen
                    l += 1
                    k += 1
                    if not (l < leftBlocks and this.compareMidKey(array, bits, k, midKey)):
                        break
                if l == leftBlocks:
                    i = this.mergeWithScrollingBufferFW(
                        array, i, j, b, i-blockLen, True)
                    this.mergeRestWithBufferFW(
                        array, i-blockLen, i, b, blockLen, True)
                else:
                    i = this.mergeWithScrollingBufferFW(
                        array, i, j, j+blockLen-1, i-blockLen, True)
                left = False
            else:
                while True:
                    j += blockLen
                    r += 1
                    k += 1
                    if not (r < t and not this.compareMidKey(array, bits, k, midKey)):
                        break
                if r == t:
                    this.shift(array, i-blockLen, i, b, False)
                    blockSwap(array, this.bufPos, b-blockLen, blockLen)
                else:
                    i = this.mergeWithScrollingBufferFW(
                        array, i, j, j+blockLen-1, i-blockLen, False)
                left = True

    def mergeBlocksLazy(this, array, a, midKey, blockQty, blockLen, lastLen, bits):
        f = a
        left = this.compareMidKey(array, bits, 0, midKey)
        i = 1
        while i < blockQty:
            if left ^ this.compareMidKey(array, bits, i, midKey):
                next = a+i*blockLen
                nextEnd = lrBinarySearch(
                    array, next, next+blockLen, array[next-1], left)
                this.mergeWithBufferBW(array, f, next, nextEnd, left)
                f = nextEnd
                left = not left
            i += 1
        if left and lastLen != 0:
            lastFrag = a+blockQty*this.blockLen
            this.mergeWithBufferBW(array, f, lastFrag, lastFrag+lastLen, left)

    def blockCycle(this, array, a, blockQty, blockLen, bits):
        i = 0
        while i < blockQty:
            k = bits.get(i)
            if k != i:
                j = i
                while True:
                    blockSwap(array, a+k*blockLen, a+j*blockLen, blockLen)
                    bits.set(j, j)
                    j = k
                    k = bits.get(k)
                    if not (k != i):
                        break
                bits.set(j, j)
            i += 1

    def kotaMerge(this, array, a, m, b1, blockLen, bits):
        i = a
        j = m
        l = a
        r = m
        t = 1
        k = 0
        while k < blockLen:
            if array[i] <= array[j]:
                array[this.bufPos+k].swap(array[i])
                i += 1
            else:
                array[this.bufPos+k].swap(array[j])
                j += 1
            k += 1
        while l < m and r < b1:
            left = i - \
                l > 0 and (
                    i-l == blockLen or array[l+blockLen-1] <= array[r+blockLen-1])
            p = l if left else r
            k = 0
            while k < blockLen:
                if j == b1 or (i < m and array[i] <= array[j]):
                    array[p].swap(array[i])
                    i += 1
                else:
                    array[p].swap(array[j])
                    j += 1
                k += 1
                p += 1
            if left:
                l = p
            else:
                r = p
            bits.set(t, (p-a)//blockLen-1)
            t += 1
        p = l if l < m else r
        blockSwap(array, this.bufPos, p, blockLen)
        bits.set(0, (p-a)//blockLen)
        while True:
            l += blockLen
            if l >= m:
                break
            bits.set(t, (l-a)//blockLen)
            t += 1
        while True:
            r += blockLen
            if r >= b1:
                break
            bits.set(t, (r-a)//blockLen)
            t += 1

    def getBlocksIndicesLazy(this, array, a, leftBlocks, rightBlocks, blockLen, indices, bits):
        l = 0
        m = leftBlocks
        r = m
        b = m+rightBlocks
        o = 0
        while l < m and r < b:
            if array[a+(l+1)*blockLen-1] <= array[a+(r+1)*blockLen-1]:
                bits.set(o, l)
                indices.set(o, l)
                l += 1
            else:
                bits.set(o, r)
                indices.set(o, r)
                r += 1
            o += 1
        while l < m:
            bits.set(o, l)
            indices.set(o, l)
            o += 1
            l += 1
        while r < b:
            bits.set(o, r)
            indices.set(o, r)
            o += 1
            r += 1

    def getBlocksIndices(this, array, a, leftBlocks, rightBlocks, blockLen, indices, bits):
        m = leftBlocks-1
        l = m
        r = leftBlocks
        b = r+rightBlocks
        o = 0
        if l != -1:
            lb = a+(l+1)*blockLen-1
            while True:
                if r == b or array[lb] <= array[a+(r+1)*blockLen-1]:
                    bits.set(o, l)
                    indices.set(o, l)
                    o += 1
                    break
                bits.set(o, r)
                indices.set(o, r)
                o += 1
                r += 1
            if l != 0:
                l = 0
                while l < m and r < b:
                    if array[a+(l+1)*blockLen-1] <= array[a+(r+1)*blockLen-1]:
                        bits.set(o, l)
                        indices.set(o, l)
                        l += 1
                    else:
                        bits.set(o, r)
                        indices.set(o, r)
                        r += 1
                    o += 1
                while l < m:
                    bits.set(o, l)
                    indices.set(o, l)
                    o += 1
                    l += 1
        while r < b:
            bits.set(o, r)
            indices.set(o, r)
            o += 1
            r += 1

    def prepareKeysLazy(this, bits, q):
        for i in range(q):
            bits.set(i, i)

    def prepareKeys(this, bits, q, leftBlocks):
        i = 0
        while i < leftBlocks-1:
            bits.set(i, i+1)
            i += 1
        bits.set(i, 0)
        i += 1
        while i < q:
            bits.set(i, i)
            i += 1

    def combine(this, array, a, m, b, bits, indices, lazy):
        if b-m <= this.bufLen:
            this.mergeWithBufferBW(array, a, m, b, True)
            return
        if this.strat1:
            blockQty = (b-a)//this.blockLen
            b1 = a+blockQty*this.blockLen
            this.kotaMerge(array, a, m, b1, this.blockLen, bits)
            this.blockCycle(array, a, blockQty, this.blockLen, bits)
            this.mergeWithBufferBW(array, a, b1, b, True)
        else:
            leftBlocks = (m-a)//this.blockLen
            rightBlocks = (b-m)//this.blockLen
            blockQty = leftBlocks+rightBlocks
            frag = (b-a)-blockQty*this.blockLen
            if lazy:
                if bits is None:
                    binaryInsertionSort(array, this.keyPos,
                                        this.keyPos+blockQty+1)
                    midKey = array[this.keyPos+leftBlocks].copy()
                    this.blockSelect(array, bits, a, leftBlocks,
                                     rightBlocks, this.blockLen)
                else:
                    midKey = leftBlocks
                    if indices is None:
                        this.prepareKeysLazy(bits, blockQty)
                        this.blockSelect(
                            array, bits, a, leftBlocks, rightBlocks, this.blockLen)
                    else:
                        this.getBlocksIndicesLazy(
                            array, a, leftBlocks, rightBlocks, this.blockLen, indices, bits)
                        this.blockCycle(array, a, blockQty,
                                        this.blockLen, indices)
                this.mergeBlocksLazy(
                    array, a, midKey, blockQty, this.blockLen, frag, bits)
            else:
                this.multiTriSwap(array, this.bufPos, m -
                                  this.blockLen, a, this.blockLen)
                leftBlocks -= 1
                blockQty -= 1
                if bits is None:
                    binaryInsertionSort(array, this.keyPos,
                                        this.keyPos+blockQty+1)
                    midKey = array[this.keyPos+leftBlocks].copy()
                    insertToRight(array, this.keyPos, this.keyPos+leftBlocks-1)
                    this.blockSelect(array, bits, a+this.blockLen,
                                     leftBlocks, rightBlocks, this.blockLen)
                else:
                    midKey = leftBlocks
                    if indices is None:
                        this.prepareKeys(bits, blockQty, leftBlocks)
                        this.blockSelect(
                            array, bits, a+this.blockLen, leftBlocks, rightBlocks, this.blockLen)
                    else:
                        this.getBlocksIndices(
                            array, a+this.blockLen, leftBlocks, rightBlocks, this.blockLen, indices, bits)
                        this.blockCycle(array, a+this.blockLen,
                                        blockQty, this.blockLen, indices)
                this.mergeBlocksWithBuf(
                    array, a, midKey, leftBlocks, rightBlocks, b, this.blockLen, bits)

    def strat2BLenCalc(this, twoR, r):
        sqrtTwoR = 1
        while sqrtTwoR*sqrtTwoR < twoR:
            sqrtTwoR *= 2
        while twoR//sqrtTwoR > r//(2*(log2(twoR//sqrtTwoR)+1)):
            sqrtTwoR *= 2
        this.blockLen = sqrtTwoR

    def noBitsBLenCalc(this, twoR):
        sqrtTwoR = 1
        while sqrtTwoR*sqrtTwoR < twoR:
            sqrtTwoR *= 2
        kCnt = twoR//sqrtTwoR+1
        if kCnt < this.keyLen:
            this.bufLen = this.keyLen-kCnt
            this.bufPos = this.keyPos+kCnt
        else:
            while twoR//sqrtTwoR+1 > this.keyLen:
                sqrtTwoR *= 2
            this.bufLen = 0
        this.blockLen = sqrtTwoR

    def resetBuf(this):
        this.bufPos = this.keyPos
        this.bufLen = this.keyLen
        this.blockLen = this.origBlockLen

    def checkValidBitArray(this, array, a, b, size):
        return a+size < b-size and array[a+size] < array[b-size]

    def adjust(this, array, a, m, b, aSub):
        frag = False
        if aSub:
            mN = a+((m-a)//this.blockLen)*this.blockLen
            bN = b-(m-mN)
            frag = mN != m
            if frag:
                this.rotate(array, mN, m, b)
            m = mN
            b = bN
        else:
            a = m-((m-a)//this.blockLen)*this.blockLen
        return a, m, b, frag

    def firstMergePart(this, array, a, m, b, bA, bB, strat2, aSub):
        if b-m <= this.bufLen:
            this.mergeWithBufferBW(array, a, m, b, True)
            return
        frag = False
        origB = b
        twoR = b-a
        if strat2:
            this.strat2BLenCalc(twoR, bB-bA)
        lazy = this.blockLen > this.bufLen
        nW = twoR//this.blockLen-int(not (lazy or this.strat1))
        w = log2(nW)+1
        size = nW*w
        if (not this.strat1) and this.checkValidBitArray(array, bA, bB, size*2):
            a, m, b, frag = this.adjust(array, a, m, b, aSub)
            bits = BitArray(array, bA, bB-size*2, nW, w)
            indices = BitArray(array, bA+size, bB-size, nW, w)
            this.combine(array, a, m, b, bits, indices, lazy)
            bits.free()
            indices.free()
        elif this.checkValidBitArray(array, bA, bB, size):
            a, m, b, frag = this.adjust(array, a, m, b, aSub)
            bits = BitArray(array, bA, bB-size, nW, w)
            this.combine(array, a, m, b, bits, None, lazy)
            bits.free()
        else:
            this.noBitsBLenCalc(twoR)
            a, m, b, frag = this.adjust(array, a, m, b, aSub)
            strat1 = this.strat1
            this.strat1 = False
            this.combine(array, a, m, b, None, None,
                         this.blockLen > this.bufLen)
            this.strat1 = strat1
            this.resetBuf()
        if frag:
            this.mergeWithBufferBW(array, a, origB, b, False)

    def firstMerge(this, array, a, m, b, strat2):
        if b-m <= this.bufLen:
            this.mergeWithBufferBW(array, a, m, b, True)
            return
        m1 = a+(m-a)//2
        m2 = lrBinarySearch(array, m, b, array[m1], True)
        m3 = m1+m2-m
        this.rotate(array, m1, m, m2)
        lAT = m3-a
        lBT = b-m3
        lA0 = m1-a
        lA1 = m3-m1
        lB0 = m2-m3
        lB1 = b-m2
        bA: int
        bB: int
        if lAT < lBT:
            if lB0 > lB1:
                bA = m3
                bB = m2
            else:
                bA = m2
                bB = b
            this.firstMergePart(array, a, m1, m3, bA, bB, strat2, True)
            this.firstMergePart(array, m3, m2, b, a, m3, strat2, False)
        else:
            if lA0 > lA1:
                bA = a
                bB = m1
            else:
                bA = m1
                bB = m3
            this.firstMergePart(array, m3, m2, b, bA, bB, strat2, False)
            this.firstMergePart(array, a, m1, m3, m3, b, strat2, True)

    def lithiumLoop(this, array, a, b):
        r = this.RUN_SIZE
        e = b-this.keyLen
        while r <= this.bufLen:
            twoR = r*2
            i = a
            while i < e-twoR:
                i += twoR
            if i+r < e:
                BufMerge2__mergeWithScrollingBufferBW(None, array, i, i+r, e)
            else:
                this.shift(array, i, e, e+r, True)
            i -= twoR
            while i >= a:
                BufMerge2__mergeWithScrollingBufferBW(None, array, i, i+r, i+twoR)
                i -= twoR
            oldR = r
            r = twoR
            twoR *= 2
            i = a+oldR
            while i+twoR < e+oldR:
                this.dualMergeFW(array, i, i+r, i+twoR, oldR)
                i += twoR
            if i+r < e+oldR:
                this.dualMergeFW(array, i, i+r, e+oldR, oldR)
            else:
                this.shift(array, i-oldR, i, e+oldR, False)
            r = twoR
        b = e
        e += this.keyLen
        strat2 = this.blockLen == 0
        twoR = r*2
        while twoR < b-a:
            i = a+twoR
            this.firstMerge(array, a, a+r, i, strat2)
            if strat2:
                this.strat2BLenCalc(twoR, twoR)
            lazy = this.blockLen > this.bufLen
            strat1 = this.strat1
            nW = twoR//this.blockLen-int(not (lazy or strat1))
            w = log2(nW)+1
            size = nW*w
            if (not strat1) and this.checkValidBitArray(array, a, a+twoR, size*2):
                bits = BitArray(array, a, a+twoR-size*2, nW, w)
                indices = BitArray(array, a+size, a+twoR-size, nW, w)
            elif this.checkValidBitArray(array, a, a+twoR, size):
                bits = BitArray(array, a, a+twoR-size, nW, w)
                indices = None
            else:
                bits = None
                indices = None
                this.strat1 = False
                this.noBitsBLenCalc(twoR)
                lazy = this.blockLen > this.bufLen
            while i < b-twoR:
                this.combine(array, i, i+r, i+twoR, bits, indices, lazy)
                i += twoR
            if i+r < b:
                this.combine(array, i, i+r, b, bits, indices, lazy)
            if bits is None:
                this.resetBuf()
                this.strat1 = strat1
            else:
                bits.free()
            if indices is not None:
                indices.free()
            r = twoR
            twoR *= 2
        this.firstMerge(array, a, a+r, b, strat2)
        single = this.bufLen <= this.SMALL_MERGE
        this.bufLen = 0
        binaryInsertionSort(array, b, e)
        if single:
            this.mergeInPlaceBW(array, a, b, e, True)
            return
        r = lrBinarySearch(array, a, b, array[e-1], False)
        this.rotate(array, r, b, e)
        d = b-r
        e -= d
        b -= d
        b0 = b+(e-b)//2
        r = lrBinarySearch(array, a, b, array[b0-1], False)
        this.rotate(array, r, b, b0)
        d = b-r
        b0 -= d
        b -= d
        this.mergeInPlaceBW(array, b0, b0+d, e, True)
        this.mergeInPlaceBW(array, a, b, b0, True)

    def inPlaceMergeSort(this, array, a, b):
        this.sortRuns(array, a, b)
        r = this.RUN_SIZE
        while r < b-a:
            twoR = r*2
            i = a
            while i < b-twoR:
                this.mergeInPlaceBW(array, i, i+r, i+twoR, True)
                i += twoR
            if i+r < b:
                this.mergeInPlaceBW(array, i, i+r, b, True)
            r = twoR

    def sort(this, array, a, b):
        n = b-a
        if n <= this.SMALL_SORT:
            this.inPlaceMergeSort(array, a, b)
            return
        sqrtn = 1
        while sqrtn*sqrtn < n:
            sqrtn *= 2
        keysFound = this.findKeys(array, a, b, sqrtn)
        if keysFound <= this.MAX_STRAT3_UNIQUE:
            this.inPlaceMergeSort(array, a, b)
            return
        this.bufPos = b-keysFound
        this.bufLen = keysFound
        this.keyLen = keysFound
        this.keyPos = this.bufPos
        this.origBlockLen = sqrtn
        if keysFound == sqrtn:
            this.blockLen = sqrtn
            this.strat1 = True
        else:
            this.blockLen = 0
            this.strat1 = False
        this.sortRuns(array, a, b-keysFound)
        this.lithiumLoop(array, a, b)


@Sort("Block Merge Sorts", "Lithium Sort", "Lithium Sort",)
def lithiumSortRun(array):
    LithiumSort().sort(array, 0, len(array))
