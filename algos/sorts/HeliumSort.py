class HeliumSort:
    RUN_SIZE = 32
    SMALL_SORT = 256
    MIN_SORTED_UNIQUE = 8
    MAX_STRAT5_UNIQUE = 8
    MIN_REV_RUN_SIZE = 8
    SMALL_MERGE = 16

    def __init__(this):
        this.buffer = None
        this.indices = None
        this.keys = None
        this.bufLen = 0
        this.bufPos = -1
        this.keyLen = 0
        this.keyPos = -1
        this.strat4A = False
        this.rotateInPlace = False

    def checkMergeBounds(this, array, a, m, b):
        if array[m-1] <= array[m]:
            return True
        elif array[a] > array[b-1]:
            this.rotate(array, a, m, b)
            return True
        return False

    def reverseRuns(this, array, a, b):
        l = a
        while l < b:
            i = l
            while i < b-1:
                if array[i] <= array[i+1]:
                    break
                i += 1
            if i-l >= this.MIN_REV_RUN_SIZE:
                reverse(array, l, i+1)
            l = i+1

    def checkSortedIdx(this, array, a, b):
        this.reverseRuns(array, a, b)
        b -= 1
        while b > a:
            if array[b] < array[b-1]:
                return b
            b -= 1
        return a

    def rotate(this, array, a, m, b):
        ip = this.buffer is None
        rl = b-m
        ll = m-a
        bl = this.bufLen if this.rotateInPlace else (
            0 if this.buffer is None else len(this.buffer))
        min_ = bl if rl != ll and min([bl, rl, ll]) > this.SMALL_MERGE else 1
        while (ll > min_ and rl > min_) or (rl < this.SMALL_MERGE and rl > 1 and ll < this.SMALL_MERGE and ll > 1):
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
        if this.rotateInPlace:
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
        else:
            if rl < ll:
                bidirArrayCopy(array, m, this.buffer, 0, rl)
                bidirArrayCopy(array, a, array, b-ll, ll)
                bidirArrayCopy(this.buffer, 0, array, a, rl)
            else:
                bidirArrayCopy(array, a, this.buffer, 0, ll)
                bidirArrayCopy(array, m, array, a, rl)
                bidirArrayCopy(this.buffer, 0, array, b-ll, ll)

    def findKeysUnsorted(this, array, a, p, b, q, to):
        n = b-p
        i = p
        while i > a and n < q:
            l = lrBinarySearch(array, p, p+n, array[i-1], True)-p
            if l == n or array[i-1] < array[p+l]:
                this.rotate(array, i, p, p+n)
                p = i-1
                insertToRight(array, i-1, p+l)
                n += 1
            i -= 1
        this.rotate(array, p, p+n, to)
        return n

    def findKeysSorted(this, array, a, b, q):
        n = 1
        p = b-1
        i = p
        while i > a and n < q:
            if array[i-1] != array[i]:
                this.rotate(array, i, p, p+n)
                p = i-1
                n += 1
            i -= 1
        if n == q:
            this.rotate(array, p, p+n, b)
        else:
            this.rotate(array, a, p, p+n)
        return n

    def findKeys(this, array, a, b, q):
        p = this.checkSortedIdx(array, a, b)
        if p == a:
            return None
        if b-p < this.MIN_SORTED_UNIQUE:
            return this.findKeysUnsorted(array, a, b-1, b, q, b)
        else:
            n = this.findKeysSorted(array, p, b, q)
            if n == q:
                return n
            return this.findKeysUnsorted(array, a, p, p+n, q, b)

    def sortRuns(this, array, a, b, p):
        if p != b:
            b = min(a+((p-a)//this.RUN_SIZE+1)*this.RUN_SIZE, b)
        speed = sortingVisualizer.getSpeed()
        sortingVisualizer.setSpeed(max(int(10*(len(array)/2048)), speed*4))
        i = a
        while i < b-this.RUN_SIZE:
            binaryInsertionSort(array, i, i+this.RUN_SIZE)
            i += this.RUN_SIZE
        if i < b:
            binaryInsertionSort(array, i, b)
        sortingVisualizer.setSpeed(speed)

    def reduceMergeBounds(this, array, a, m, b):
        return (lrBinarySearch(array, a, m-1, array[m], False), lrBinarySearch(array, m, b, array[m-1], True))

    def mergeInPlaceFW(this, array, a, m, b, left):
        s = a
        l = m
        while s < l and l < b:
            cmp = compareValues(array[s], array[l])
            if cmp > 0 if left else cmp >= 0:
                p = lrBinarySearch(array, l, b, array[s], left)
                this.rotate(array, s, l, p)
                s += p-l
                l = p
            else:
                s += 1

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

    def mergeInPlace(this, array, a, m, b, left, check):
        if check:
            if this.checkMergeBounds(array, a, m, b):
                return
            a, b = this.reduceMergeBounds(array, a, m, b)
        if m-a > b-m:
            this.mergeInPlaceBW(array, a, m, b, left)
        else:
            this.mergeInPlaceFW(array, a, m, b, left)

    def mergeWithBufferFW(this, array, a, m, b, buf, left):
        ll = m-a
        blockSwap(array, a, buf, ll)
        l = buf
        r = m
        o = a
        e = buf+ll
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

    def mergeWithBufferBW(this, array, a, m, b, buf, left):
        rl = b-m
        backwardBlockSwap(array, m, buf, rl)
        l = m-1
        r = buf+rl-1
        o = b-1
        while l >= a and r >= buf:
            cmp = compareValues(array[r], array[l])
            if cmp >= 0 if left else cmp > 0:
                array[o].swap(array[r])
                r -= 1
            else:
                array[o].swap(array[l])
                l -= 1
            o -= 1
        while r >= buf:
            array[o].swap(array[r])
            o -= 1
            r -= 1

    def mergeWithBuffer(this, array, a, m, b, buf, left):
        if this.checkMergeBounds(array, a, m, b):
            return
        a, b = this.reduceMergeBounds(array, a, m, b)
        ll = m-a
        rl = b-m
        if ll > rl:
            if rl <= this.SMALL_MERGE:
                this.mergeInPlaceBW(array, a, m, b, left)
            else:
                this.mergeWithBufferBW(array, a, m, b, buf, left)
        else:
            if ll <= this.SMALL_MERGE:
                this.mergeInPlaceFW(array, a, m, b, left)
            else:
                this.mergeWithBufferFW(array, a, m, b, buf, left)

    def mergeOOPFW(this, array, a, m, b, left):
        ll = m-a
        arrayCopy(array, a, this.buffer, 0, ll)
        l = 0
        r = m
        o = a
        e = ll
        while l < e and r < b:
            cmp = compareValues(this.buffer[l], array[r])
            if cmp <= 0 if left else cmp < 0:
                array[o].write(this.buffer[l])
                l += 1
            else:
                array[o].write(array[r])
                r += 1
            o += 1
        while l < e:
            array[o].write(this.buffer[l])
            o += 1
            l += 1

    def mergeOOPBW(this, array, a, m, b, left):
        rl = b-m
        arrayCopy(array, m, this.buffer, 0, rl)
        l = m-1
        r = rl-1
        o = b-1
        while l >= a and r >= 0:
            cmp = compareValues(this.buffer[r], array[l])
            if cmp >= 0 if left else cmp > 0:
                array[o].write(this.buffer[r])
                r -= 1
            else:
                array[o].write(array[l])
                l -= 1
            o -= 1
        while r >= 0:
            array[o].write(this.buffer[r])
            o -= 1
            r -= 1

    def mergeOOP(this, array, a, m, b, left):
        if this.checkMergeBounds(array, a, m, b):
            return
        a, b = this.reduceMergeBounds(array, a, m, b)
        ll = m-a
        rl = b-m
        if ll > rl:
            if rl <= this.SMALL_MERGE:
                this.mergeInPlaceBW(array, a, m, b, left)
            else:
                this.mergeOOPBW(array, a, m, b, left)
        else:
            if ll <= this.SMALL_MERGE:
                this.mergeInPlaceFW(array, a, m, b, left)
            else:
                this.mergeOOPFW(array, a, m, b, left)

    def optiSmartMerge(this, array, a, m, b, buf, left):
        ll = m-a
        rl = b-m
        if ll > rl:
            if rl <= this.SMALL_MERGE:
                this.mergeInPlaceBW(array, a, m, b, left)
                return True
            if this.buffer is not None and rl < len(this.buffer):
                this.mergeOOPBW(array, a, m, b, left)
            elif buf != -1 and rl <= this.bufLen:
                this.mergeWithBufferBW(array, a, m, b, buf, left)
            else:
                return False
        else:
            if ll <= this.SMALL_MERGE:
                this.mergeInPlaceFW(array, a, m, b, left)
                return True
            if this.buffer is not None and ll <= len(this.buffer):
                this.mergeOOPFW(array, a, m, b, left)
            elif buf != -1 and ll <= this.bufLen:
                this.mergeWithBufferFW(array, a, m, b, buf, left)
            else:
                return False
        return True

    def getBlocksIndices(this, array, a, leftBlocks, rightBlocks, blockLen):
        l = 0
        m = leftBlocks
        r = m
        b = m+rightBlocks
        o = 0
        while l < m and r < b:
            if array[a+(l+1)*blockLen-1] <= array[a+(r+1)*blockLen-1]:
                this.indices[o].write(l)
                l += 1
            else:
                this.indices[o].write(r)
                r += 1
            o += 1
        while l < m:
            this.indices[o].write(l)
            o += 1
            l += 1
        while r < b:
            this.indices[o].write(r)
            o += 1
            r += 1

    def blockCycle(this, array, a, leftBlocks, rightBlocks, blockLen):
        total = leftBlocks+rightBlocks
        i = 0
        while i < total:
            if this.indices[i] != i:
                arrayCopy(array, a+i*blockLen, this.buffer, 0, blockLen)
                j = i
                next = this.indices[i].readInt()
                while True:
                    bidirArrayCopy(array, a+next*blockLen,
                                   array, a+j*blockLen, blockLen)
                    this.indices[j].write(j)
                    j = next
                    next = this.indices[next].readInt()
                    if not (next != i):
                        break
                arrayCopy(this.buffer, 0, array, a+j*blockLen, blockLen)
                this.indices[j].write(j)
            i += 1

    def blockSelectInPlace(this, array, stKey, a, leftBlocks, rightBlocks, blockLen):
        i1 = stKey
        tm = stKey+leftBlocks
        j1 = tm
        k = stKey
        tb = tm+rightBlocks
        while k < j1 and j1 < tb:
            if array[a+(i1-stKey+1)*blockLen-1] <= array[a+(j1-stKey+1)*blockLen-1]:
                if i1 > k:
                    blockSwap(array, a+(k-stKey)*blockLen,
                              a+(i1-stKey)*blockLen, blockLen)
                array[k].swap(array[i1])
                k += 1
                i1 = k
                i = max(k+1, tm)
                while i < j1:
                    if array[i] < array[i1]:
                        i1 = i
                    i += 1
            else:
                blockSwap(array, a+(k-stKey)*blockLen,
                          a+(j1-stKey)*blockLen, blockLen)
                array[k].swap(array[j1])
                j1 += 1
                if i1 == k:
                    i1 = j1-1
                k += 1
        while k < j1-1:
            if i1 > k:
                blockSwap(array, a+(k-stKey)*blockLen,
                          a+(i1-stKey)*blockLen, blockLen)
            array[k].swap(array[i1])
            k += 1
            i1 = k
            i = k+1
            while i < j1:
                if array[i] < array[i1]:
                    i1 = i
                i += 1

    def blockSelectOOP(this, array, a, leftBlocks, rightBlocks, blockLen):
        i1 = 0
        tm = leftBlocks
        j1 = tm
        k = 0
        tb = tm+rightBlocks
        while k < j1 and j1 < tb:
            if array[a+(i1+1)*blockLen-1] <= array[a+(j1+1)*blockLen-1]:
                if i1 > k:
                    blockSwap(array, a+k*blockLen, a+i1*blockLen, blockLen)
                this.keys[k].swap(this.keys[i1])
                k += 1
                i1 = k
                i = max(k+1, tm)
                while i < j1:
                    if this.keys[i] < this.keys[i1]:
                        i1 = i
                    i += 1
            else:
                blockSwap(array, a+k*blockLen, a+j1*blockLen, blockLen)
                this.keys[k].swap(this.keys[j1])
                j1 += 1
                if i1 == k:
                    i1 = j1-1
                k += 1
        while k < j1-1:
            if i1 > k:
                blockSwap(array, a+k*blockLen, a+i1*blockLen, blockLen)
            this.keys[k].swap(this.keys[i1])
            k += 1
            i1 = k
            i = k+1
            while i < j1:
                if this.keys[i] < this.keys[i1]:
                    i1 = i
                i += 1

    def smartMerge(this, array, a, m, b, left):
        if this.optiSmartMerge(array, a, m, b, this.bufPos, left):
            return
        this.mergeInPlace(array, a, m, b, left, False)

    def mergeBlocks(this, array, a, midKey, blockQty, blockLen, lastLen, stKey, keys):
        f = a
        left = keys[stKey] < midKey
        i = 1
        while i < blockQty:
            if left ^ (keys[stKey+i] < midKey):
                next = a+i*blockLen
                nextEnd = lrBinarySearch(
                    array, next, next+blockLen, array[next-1], left)
                this.smartMerge(array, f, next, nextEnd, left)
                f = nextEnd
                left = not left
            i += 1
        if left and lastLen != 0:
            lastFrag = a+blockQty*this.blockLen
            this.smartMerge(array, f, lastFrag, lastFrag+lastLen, left)

    def prepareOOPKeys(this, blockQty):
        i = 0
        while i < blockQty:
            this.keys[i].write(i)
            i += 1

    def combineReduce(this, array, a, m, b):
        if this.checkMergeBounds(array, a, m, b):
            return None, None
        oldA = a
        a, b = this.reduceMergeBounds(array, a, m, b)
        if this.optiSmartMerge(array, a, m, b, this.bufPos, True):
            return None, None
        a = max(oldA, m-((m-a)//this.blockLen+1)*this.blockLen)
        return a, b

    def hydrogenCombine(this, array, a, m, b):
        a, b = this.combineReduce(array, a, m, b)
        if a is None:
            return
        leftBlocks = (m-a)//this.blockLen
        rightBlocks = (b-m)//this.blockLen
        blockQty = leftBlocks+rightBlocks
        frag = (b-a)-blockQty*this.blockLen
        this.getBlocksIndices(array, a, leftBlocks, rightBlocks, this.blockLen)
        arrayCopy(this.indices, 0, this.keys, 0, blockQty)
        this.blockCycle(array, a, leftBlocks, rightBlocks, this.blockLen)
        this.mergeBlocks(array, a, leftBlocks, blockQty,
                         this.blockLen, frag, 0, this.keys)

    def heliumCombine(this, array, a, m, b):
        a, b = this.combineReduce(array, a, m, b)
        if a is None:
            return
        leftBlocks = (m-a)//this.blockLen
        rightBlocks = (b-m)//this.blockLen
        blockQty = leftBlocks+rightBlocks
        frag = (b-a)-blockQty*this.blockLen
        if this.keys is None:
            binaryInsertionSort(array, this.keyPos, this.keyPos+blockQty+1)
            midKey = array[this.keyPos+leftBlocks].readInt()
            this.blockSelectInPlace(
                array, this.keyPos, a, leftBlocks, rightBlocks, this.blockLen)
            this.mergeBlocks(array, a, midKey, blockQty,
                             this.blockLen, frag, this.keyPos, array)
        else:
            this.prepareOOPKeys(blockQty)
            this.blockSelectOOP(array, a, leftBlocks,
                                rightBlocks, this.blockLen)
            this.mergeBlocks(array, a, leftBlocks, blockQty,
                             this.blockLen, frag, 0, this.keys)

    def uraniumLoop(this, array, a, b):
        r = this.RUN_SIZE
        while r < b-a:
            twoR = r*2
            i = a
            while i < b-twoR:
                this.mergeOOP(array, i, i+r, i+twoR, True)
                i += twoR
            if i+r < b:
                this.mergeOOP(array, i, i+r, b, True)
            r = twoR

    def hydrogenLoop(this, array, a, b):
        r = this.RUN_SIZE
        while r < len(this.buffer):
            twoR = r*2
            i = a
            while i < b-twoR:
                this.mergeOOP(array, i, i+r, i+twoR, True)
                i += twoR
            if i+r < b:
                this.mergeOOP(array, i, i+r, b, True)
            r = twoR
        while r < b-a:
            twoR = r*2
            i = a
            while i < b-twoR:
                this.hydrogenCombine(array, i, i+r, i+twoR)
                i += twoR
            if i+r < b:
                this.hydrogenCombine(array, i, i+r, b)
            r = twoR

    def heliumLoop(this, array, a, b):
        r = this.RUN_SIZE
        if this.buffer is not None:
            while r < len(this.buffer):
                twoR = r*2
                i = a
                while i < b-twoR:
                    this.mergeOOP(array, i, i+r, i+twoR, True)
                    i += twoR
                if i+r < b:
                    this.mergeOOP(array, i, i+r, b, True)
                r = twoR
        while r <= this.bufLen:
            twoR = r*2
            i = a
            while i < b-twoR:
                this.mergeWithBuffer(array, i, i+r, i+twoR, this.bufPos, True)
                i += twoR
            if i+r < b:
                this.mergeWithBuffer(array, i, i+r, b, this.bufPos, True)
            r = twoR
        strat4 = this.blockLen == 0 or this.strat4A
        while r < b-a:
            twoR = r*2
            if strat4:
                if this.strat4A:
                    bLen = this.blockLen
                    while twoR//bLen+1 > this.keyLen:
                        bLen *= 2
                    this.blockLen = bLen
                else:
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
            i = a
            while i < b-twoR:
                this.heliumCombine(array, i, i+r, i+twoR)
                i += twoR
            if i+r < b:
                if strat4 and b-i-r <= this.keyLen:
                    this.bufPos = this.keyPos
                    this.bufLen = this.keyLen
                this.heliumCombine(array, i, i+r, b)
            r = twoR
        if this.keyLen != 0 or this.bufLen != 0:
            s = this.keyPos
            e: int
            if strat4:
                e = s+this.keyLen
            else:
                e = s+this.keyLen+this.bufLen
            this.bufLen = 0
            binaryInsertionSort(array, s, e)
            a, e = this.reduceMergeBounds(array, a, s, e)
            if not this.optiSmartMerge(array, a, s, e, -1, True):
                this.mergeInPlace(array, a, s, e, True, False)

    def inPlaceMergeSort(this, array, a, b, check):
        if check:
            p = this.checkSortedIdx(array, a, b)
            if p == a:
                return
            this.sortRuns(array, a, b, p)
        else:
            this.sortRuns(array, a, b, b)
        r = this.RUN_SIZE
        while r < b-a:
            twoR = r*2
            i = a
            while i < b-twoR:
                this.mergeInPlace(array, i, i+r, i+twoR, True, True)
                i += twoR
            if i+r < b:
                this.mergeInPlace(array, i, i+r, b, True, True)
            r = twoR

    def sort(this, array, a, b, mem):
        n = b-a
        if n <= this.SMALL_SORT:
            this.inPlaceMergeSort(array, a, b, True)
            return
        if mem >= n//2 or mem == -1:
            if mem == -1:
                mem = n//2
            p = this.checkSortedIdx(array, a, b)
            if p == a:
                return
            this.sortRuns(array, a, b, p)
            this.buffer = sortingVisualizer.createValueArray(mem)
            this.uraniumLoop(array, a, b)
            return
        sqrtn = 1
        while sqrtn*sqrtn < n:
            sqrtn *= 2
        keySize = n//sqrtn
        if mem >= sqrtn+2*keySize or mem == -2:
            if mem == -2:
                mem = sqrtn+2*keySize
            elif mem != sqrtn+2*keySize:
                while sqrtn+2*n//sqrtn <= mem:
                    sqrtn *= 2
                sqrtn //= 2
                keySize = n//sqrtn
            p = this.checkSortedIdx(array, a, b)
            if p == a:
                return
            this.sortRuns(array, a, b, p)
            this.buffer = sortingVisualizer.createValueArray(mem-2*keySize)
            this.indices = sortingVisualizer.createValueArray(keySize)
            this.keys = sortingVisualizer.createValueArray(keySize)
            sortingVisualizer.setNonOrigAux([this.indices, this.keys])
            this.blockLen = sqrtn
            this.hydrogenLoop(array, a, b)
            return
        if mem >= sqrtn+keySize or mem == -3:
            if mem == -3:
                mem = sqrtn+keySize
            elif mem != sqrtn+keySize:
                while sqrtn+n//sqrtn <= mem:
                    sqrtn *= 2
                sqrtn //= 2
                keySize = n//sqrtn
            p = this.checkSortedIdx(array, a, b)
            if p == a:
                return
            this.sortRuns(array, a, b, p)
            this.buffer = sortingVisualizer.createValueArray(mem-keySize)
            this.keys = sortingVisualizer.createValueArray(keySize)
            sortingVisualizer.setNonOrigAux([this.keys])
            this.blockLen = sqrtn
            this.heliumLoop(array, a, b)
            return
        if mem >= sqrtn or mem == -4:
            if mem == -4:
                mem = sqrtn
            elif mem != sqrtn:
                while sqrtn <= mem:
                    sqrtn *= 2
                sqrtn //= 2
                keySize = n//sqrtn
            this.buffer = sortingVisualizer.createValueArray(mem)
            keysFound = this.findKeys(array, a, b, keySize)
            if keysFound is None:
                return
            if keysFound <= this.MAX_STRAT5_UNIQUE:
                this.inPlaceMergeSort(array, a, b, False)
                return
            this.sortRuns(array, a, b-keysFound, b-keysFound)
            this.keyLen = keysFound
            this.keyPos = b-keysFound
            this.blockLen = sqrtn
            this.strat4A = keysFound != keySize
            this.heliumLoop(array, a, b-keysFound)
            return
        if mem > 0:
            this.buffer = sortingVisualizer.createValueArray(mem)
        ideal = sqrtn+keySize
        keysFound = this.findKeys(array, a, b, ideal)
        if keysFound is None:
            return
        if keysFound <= this.MAX_STRAT5_UNIQUE:
            this.inPlaceMergeSort(array, a, b, False)
            return
        this.sortRuns(array, a, b-keysFound, b-keysFound)
        if keysFound == ideal:
            this.blockLen = sqrtn
            this.bufLen = sqrtn
            this.bufPos = b-sqrtn
            this.keyLen = keySize
            this.keyPos = this.bufPos-keySize
        else:
            this.blockLen = 0
            this.bufLen = keysFound
            this.bufPos = b-keysFound
            this.keyLen = keysFound
            this.keyPos = b-keysFound
        this.rotateInPlace = this.bufLen > (mem*2)
        this.heliumLoop(array, a, b-keysFound)


@Sort("Block Merge Sorts", "Helium Sort", "Helium Sort")
def heliumGenSortRun(array):
    mem = sortingVisualizer.getUserInput(
        "Insert memory size (or -4 .. -1 for default modes)", "0", parseInt)
    HeliumSort().sort(array, 0, len(array), mem)


@Sort("Merge Sorts", "Uranium Sort (Helium strategy 1)", "Uranium Sort")
def uraniumSortRun(array):
    HeliumSort().sort(array, 0, len(array), -1)


@Sort("Block Merge Sorts", "Hydrogen Sort (Helium strategy 2)", "Hydrogen Sort")
def hydrogenSortRun(array):
    HeliumSort().sort(array, 0, len(array), -2)
