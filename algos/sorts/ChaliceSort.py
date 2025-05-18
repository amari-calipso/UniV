class ChaliceSort:
    def __init__(this, rot):
        if rot is None:
            this.rotate = UniV_getUserRotation("Select rotation algorithm (default: Cycle Reverse)").indexed
        else:
            this.rotate = sortingVisualizer.getRotationByName(rot).indexed
        this.aux = None

    def ceilCbrt(this, n):
        a = 0
        b = 11
        while a < b:
            m = (a+b)//2
            if (1 << 3*m) >= n:
                b = m
            else:
                a = m+1
        return 1 << a

    def calcKeys(this, bLen, n):
        a = 1
        b = n//4
        while a < b:
            m = (a+b)//2
            if (n-4*m-1)//bLen-2 < m:
                b = m
            else:
                a = m+1
        return a

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

    def laziestSortExt(this, array, a, b):
        i = a
        s = len(this.aux)
        while i < b:
            j = min(b, i+s)
            binaryInsertionSort(array, i, j)
            if i > a:
                this.mergeBWExt(array, a, i, j)
            i += s

    def findKeysSm(this, array, a, b, a1, b1, full, n):
        rotate = this.rotate
        p = a
        pEnd = 0
        if full:
            while p < b:
                loc = lrBinarySearch(array, a1, b1, array[p], True)
                if loc == b1 or array[p] != array[loc]:
                    pEnd = p+1
                    break
                p += 1
            if pEnd != 0:
                i = pEnd
                while i < b and pEnd-p < n:
                    loc = lrBinarySearch(array, a1, b1, array[i], True)
                    if loc == b1 or array[i] != array[loc]:
                        loc = lrBinarySearch(array, p, pEnd, array[i], True)
                        if loc == pEnd or array[i] != array[loc]:
                            rotate(array, p, pEnd, i)
                            len1 = i-pEnd
                            p += len1
                            loc += len1
                            pEnd = i+1
                            insertToLeft(array, i, loc)
                    i += 1
            else:
                pEnd = p
        else:
            pEnd = p+1
            i = pEnd
            while i < b and pEnd-p < n:
                loc = lrBinarySearch(array, p, pEnd, array[i], True)
                if loc == pEnd or array[i] != array[loc]:
                    rotate(array, p, pEnd, i)
                    len1 = i-pEnd
                    p += len1
                    loc += len1
                    pEnd = i+1
                    insertToLeft(array, i, loc)
                i += 1
        return [p, pEnd]

    def findKeys(this, array, a, b, n, s):
        rotate = this.rotate
        p: int
        pEnd: int
        p, pEnd = this.findKeysSm(array, a, b, 0, 0, False, min(n, s))
        if s < n and pEnd-p == s:
            n -= s
            while True:
                t = this.findKeysSm(array, pEnd, b, p, pEnd, True, min(s, n))
                keys = t[1]-t[0]
                if keys == 0:
                    break
                if keys < s or n == s:
                    rotate(array, pEnd, t[0], t[1])
                    t[0] = pEnd
                    pEnd += keys
                    this.mergeBWExt(array, p, t[0], pEnd)
                    break
                else:
                    rotate(array, p, pEnd, t[0])
                    p += t[0]-pEnd
                    pEnd = t[1]
                    this.mergeBWExt(array, p, t[0], pEnd)
                n -= s
        rotate(array, a, p, pEnd)
        return pEnd-p

    def findBitsSm(this, array, a, b, a1, bw, n):
        rotate = this.rotate
        p = a
        cmp = -1 if bw else 1
        pEnd: int
        while p < b and compareValues(array[p], array[a1]) != cmp:
            p += 1
        a1 += 1
        if p < b:
            pEnd = p+1
            i = pEnd
            while i < b and pEnd-p < n:
                if compareValues(array[i], array[a1]) == cmp:
                    rotate(array, p, pEnd, i)
                    p += i-pEnd
                    pEnd = i+1
                    a1 += 1
                i += 1
        else:
            pEnd = p
        return [p, pEnd]

    def findBits(this, array, a, b, n, s):
        rotate = this.rotate
        this.laziestSortExt(array, a, a+n)
        a0 = a
        a1 = a+n
        c = 0
        c0 = 0
        i = 0
        while c < n and i < 2:
            p = a1
            pEnd = p
            while True:
                t = this.findBitsSm(array, pEnd, b, a0, i == 1, min(s, n-c))
                bits = t[1]-t[0]
                if bits == 0:
                    break
                a0 += bits
                c += bits
                if bits < s or c == n:
                    rotate(array, pEnd, t[0], t[1])
                    t[0] = pEnd
                    pEnd += bits
                    break
                else:
                    rotate(array, p, pEnd, t[0])
                    p += t[0]-pEnd
                    pEnd = t[1]
            rotate(array, a1, p, pEnd)
            a1 += pEnd-p
            if i == 0:
                c0 = c
            i += 1
        if c < n:
            return -1
        else:
            blockSwap(array, a+c0, a+n+c0, n-c0)
            return c0

    def bitReversal(this, array, a, b):
        len = b-a
        m = 0
        d1 = len >> 1
        d2 = d1+(d1 >> 1)
        i = 1
        while i < len-1:
            j = d1
            k = i
            n = d2
            while (k & 1) == 0:
                j -= n
                k >>= 1
                n >>= 1
            m += j
            if m > i:
                array[a+i].swap(array[a+m])
            i += 1

    def unshuffle(this, array, a, b):
        rotate = this.rotate
        len = (b-a) >> 1
        c = 0
        n = 2
        while len > 0:
            if (len & 1) == 1:
                a1 = a+c
                this.bitReversal(array, a1, a1+n)
                this.bitReversal(array, a1, a1+n//2)
                this.bitReversal(array, a1+n//2, a1+n)
                rotate(array, a+c//2, a1, a1+n//2)
                c += n
            len >>= 1
            n *= 2

    def redistBuffer(this, array, a, m, b):
        rotate = this.rotate
        s = len(this.aux)
        while m-a > s and m < b:
            i = lrBinarySearch(array, m, b, array[a+s], True)
            rotate(array, a+s, m, i)
            t = i-m
            m = i
            this.mergeFWExt(array, a, a+s, m)
            a += t+s
        if m < b:
            this.mergeFWExt(array, a, m, b)

    def shiftBW(this, array, a, m, b):
        while m > a:
            b -= 1
            m -= 1
            array[b].swap(array[m])

    def dualMergeBW(this, array, a, m, b, p):
        i = m-1
        b -= 1
        while p > b+1 and b >= m:
            p -= 1
            if array[b] >= array[i]:
                array[p].swap(array[b])
                b -= 1
            else:
                array[p].swap(array[i])
                i -= 1
        if b < m:
            this.shiftBW(array, a, i+1, p)
        else:
            i += 1
            b += 1
            p = m-(i-a)
            while a < i and m < b:
                if array[a] <= array[m]:
                    array[p].swap(array[a])
                    a += 1
                else:
                    array[p].swap(array[m])
                    m += 1
                p += 1
            while a < i:
                array[p].swap(array[a])
                p += 1
                a += 1

    def shiftBWExt(this, array, a, m, b):
        while m > a:
            b -= 1
            m -= 1
            array[b].write(array[m])

    def dualMergeBWExt(this, array, a, m, b, p):
        i = m-1
        b -= 1
        while p > b+1 and b >= m:
            p -= 1
            if array[b] >= array[i]:
                array[p].write(array[b])
                b -= 1
            else:
                array[p].write(array[i])
                i -= 1
        if b < m:
            this.shiftBWExt(array, a, i+1, p)
        else:
            i += 1
            b += 1
            p = m-(i-a)
            while a < i and m < b:
                if array[a] <= array[m]:
                    array[p].write(array[a])
                    a += 1
                else:
                    array[p].write(array[m])
                    m += 1
                p += 1
            while a < i:
                array[p].write(array[a])
                p += 1
                a += 1

    def smartMerge(this, array, p, a, m, rev):
        i = m
        cmp = int(not rev)
        while a < m:
            if compareValues(array[a], array[i]) < cmp:
                array[p].write(array[a])
                a += 1
            else:
                array[p].write(array[i])
                i += 1
            p += 1
        return i

    def shiftFWExt(this, array, a, m, b):
        while m < b:
            array[a].write(array[m])
            a += 1
            m += 1

    def smartTailMerge(this, array, p, a, m, b):
        i = m
        bLen = len(this.aux)
        while a < m and i < b:
            if array[a] <= array[i]:
                array[p].write(array[a])
                a += 1
            else:
                array[p].write(array[i])
                i += 1
            p += 1
        if a < m:
            if a > p:
                this.shiftFWExt(array, p, a, m)
            bidirArrayCopy(this.aux, 0, array, b-bLen, bLen)
        else:
            a = 0
            while a < bLen and i < b:
                if this.aux[a] <= array[i]:
                    array[p].write(this.aux[a])
                    a += 1
                else:
                    array[p].write(array[i])
                    i += 1
                p += 1
            while a < bLen:
                array[p].write(this.aux[a])
                p += 1
                a += 1

    def blockCycle(this, array, a, t, tIdx, tLen, bLen):
        for i in range(tLen-1):
            if array[t+i] > array[tIdx+i] or (i > 0 and array[t+i] < array[tIdx+i-1]):
                bidirArrayCopy(array, a+i*bLen, array, a-bLen, bLen)
                val = i
                next = lrBinarySearch(
                    array, tIdx, tIdx+tLen, array[t+i], True)-tIdx
                while True:
                    bidirArrayCopy(array, a+next*bLen, array, a+val*bLen, bLen)
                    array[t+i].swap(array[t+next])
                    val = next
                    next = lrBinarySearch(
                        array, tIdx, tIdx+tLen, array[t+i], True)-tIdx
                    if not (next != i):
                        break
                bidirArrayCopy(array, a-bLen, array, a+val*bLen, bLen)

    def blockMerge(this, array, a, m, b, tl, tLen, t, tIdx, bp1, bp2, bLen):
        if b-m <= bLen:
            this.mergeBWExt(array, a, m, b)
            return
        insertToLeft(array, t+tl-1, t)
        i = a+bLen-1
        j = m+bLen-1
        ti = t
        tj = t+tl
        tp = tIdx
        while ti < t+tl and tj < t+tLen:
            if array[i] <= array[j]:
                array[tp].swap(array[ti])
                ti += 1
                i += bLen
            else:
                array[tp].swap(array[tj])
                array[bp1].swap(array[bp2])
                tj += 1
                j += bLen
            tp += 1
            bp1 += 1
            bp2 += 1
        while ti < t+tl:
            array[tp].swap(array[ti])
            tp += 1
            ti += 1
            bp1 += 1
            bp2 += 1
        while tj < t+tLen:
            array[tp].swap(array[tj])
            array[bp1].swap(array[bp2])
            tp += 1
            tj += 1
            bp1 += 1
            bp2 += 1
        t ^= tIdx
        tIdx ^= t
        t ^= tIdx
        MaxHeapSort().sort(array, tIdx, tIdx+tLen)
        bidirArrayCopy(array, m-bLen, this.aux, 0, bLen)
        bidirArrayCopy(array, a, array, m-bLen, bLen)
        this.blockCycle(array, a+bLen, t, tIdx, tLen, bLen)
        blockSwap(array, t, tIdx, tLen)
        bp1 -= tLen
        bp2 -= tLen
        f = a+bLen
        a1 = f
        bp3 = bp2+tLen
        rev = array[bp1] > array[bp2]
        while True:
            while True:
                if rev:
                    array[bp1].swap(array[bp2])
                bp1 += 1
                bp2 += 1
                a1 += bLen
                if not (bp2 < bp3 and compareValues(array[bp1], array[bp2]) == (1 if rev else -1)):
                    break
            if bp2 == bp3:
                this.smartTailMerge(array, f-bLen, f, (f if rev else a1), b)
                return
            f = this.smartMerge(array, f-bLen, f, a1, rev)
            rev = not rev

    def blockCycleEasy(this, array, a, t, tIdx, tLen, bLen):
        for i in range(tLen-1):
            if array[t+i] > array[tIdx+i] or (i > 0 and array[t+i] < array[tIdx+i-1]):
                next = lrBinarySearch(array, tIdx, tIdx+tLen, array[t+i], True)-tIdx
                while True:
                    blockSwap(array, a+i*bLen, a+next*bLen, bLen)
                    array[t+i].swap(array[t+next])
                    next = lrBinarySearch(
                        array, tIdx, tIdx+tLen, array[t+i], True)-tIdx
                    if not (next != i):
                        break

    def inPlaceMergeBW(this, array, a, m, b, rev):
        rotate = this.rotate
        f = lrBinarySearch(array, m, b, array[m-1], not rev)
        b = f
        while b > m and m > a:
            i = lrBinarySearch(array, a, m, array[b-1], rev)
            rotate(array, i, m, b)
            t = m-i
            m = i
            b -= t+1
            if m == a:
                break
            b = lrBinarySearch(array, m, b, array[m-1], not rev)
        return f

    def inPlaceMerge(this, array, a, m, b):
        rotate = this.rotate
        while a < m and m < b:
            a = lrBinarySearch(array, a, m, array[m], False)
            if a == m:
                return
            i = lrBinarySearch(array, m, b, array[a], True)
            rotate(array, a, m, i)
            t = i-m
            m = i
            a += t+1

    def blockMergeEasy(this, array, a, m, b, lenA, lenB, tl, tLen, t, tIdx, bp1, bp2, bLen):
        if b-m <= bLen:
            this.inPlaceMergeBW(array, a, m, b, False)
            return
        a1 = a+lenA
        b1 = b-lenB
        i = a1+bLen-1
        j = m+bLen-1
        ti = tIdx
        tj = tIdx+tl
        tp = t
        while ti < tIdx+tl and tj < tIdx+tLen:
            if array[i] <= array[j]:
                array[ti].swap(array[tp])
                ti += 1
                i += bLen
            else:
                array[tj].swap(array[tp])
                array[bp1].swap(array[bp2])
                tj += 1
                j += bLen
            tp += 1
            bp1 += 1
            bp2 += 1
        while ti < tIdx+tl:
            array[ti].swap(array[tp])
            ti += 1
            tp += 1
            bp1 += 1
            bp2 += 1
        while tj < tIdx+tLen:
            array[tj].swap(array[tp])
            array[bp1].swap(array[bp2])
            tj += 1
            tp += 1
            bp1 += 1
            bp2 += 1
        t ^= tIdx
        tIdx ^= t
        t ^= tIdx
        MaxHeapSort().sort(array, tIdx, tIdx+tLen)
        this.blockCycleEasy(array, a1, t, tIdx, tLen, bLen)
        blockSwap(array, t, tIdx, tLen)
        bp1 -= tLen
        bp2 -= tLen
        f = a1
        a2 = f
        bp3 = bp2+tLen
        rev = array[bp1] > array[bp2]
        while True:
            while True:
                if rev:
                    array[bp1].swap(array[bp2])
                bp1 += 1
                bp2 += 1
                a2 += bLen
                if not (bp2 < bp3 and compareValues(array[bp1], array[bp2]) == (1 if rev else -1)):
                    break
            if bp2 == bp3:
                if not rev:
                    this.inPlaceMergeBW(array, a1, b1, b, False)
                this.inPlaceMerge(array, a, a1, b)
                return
            f = this.inPlaceMergeBW(array, f, a2, a2+bLen, rev)
            rev = not rev

    def lazyStable(this, array, a, b):
        j = 1
        while j < b - a:
            i = a+j
            while i < b:
                this.inPlaceMergeBW(array, i-j, i, min(i+j, b))
                i += 2*j
            j *= 2

    def mergeWithBufFWExt(this, array, a, m, b, p):
        i = m
        while a < m and i < b:
            if array[a] <= array[i]:
                array[p].write(array[a])
                a += 1
            else:
                array[p].write(array[i])
                i += 1
            p += 1
        if a > p:
            this.shiftFWExt(array, p, a, m)
        this.shiftFWExt(array, p, i, b)

    def mergeWithBufBWExt(this, array, a, m, b, p):
        i = m-1
        b -= 1
        while b >= m and i >= a:
            p -= 1
            if array[b] >= array[i]:
                array[p].write(array[b])
                b -= 1
            else:
                array[p].write(array[i])
                i -= 1
        if p > b:
            this.shiftBWExt(array, m, b+1, p)
        this.shiftBWExt(array, a, i+1, p)

    def shiftFW(this, array, a, m, b):
        while m < b:
            array[a].swap(array[m])
            a += 1
            m += 1

    def mergeWithBufFW(this, array, a, m, b, p):
        i = m
        while a < m and i < b:
            if array[a] <= array[i]:
                array[p].swap(array[a])
                a += 1
            else:
                array[p].swap(array[i])
                i += 1
            p += 1
        if a > p:
            this.shiftFW(array, p, a, m)
        this.shiftFW(array, p, i, b)

    def mergeWithBufBW(this, array, a, m, b, p):
        i = m-1
        b -= 1
        while b >= m and i >= a:
            p -= 1
            if array[b] >= array[i]:
                array[p].swap(array[b])
                b -= 1
            else:
                array[p].swap(array[i])
                i -= 1
        if p > b:
            this.shiftBW(array, m, b+1)
        this.shiftBW(array, a, i+1, p)

    def sort(this, array, a, b):
        rotate = this.rotate
        n = b-a
        if n < 128:
            if n < 32:
                binaryInsertionSort(array, a, b)
            else:
                this.lazyStable(array, a, b)
            return
        cbrt = 2*this.ceilCbrt(n//4)
        bLen = 2*cbrt
        kLen = this.calcKeys(bLen, n)
        this.aux = sortingVisualizer.createValueArray(bLen)
        keys = this.findKeys(array, a, b, 2*kLen, cbrt)
        if keys < 8:
            this.lazyStable(array, a, b)
            return
        elif keys < 2*kLen:
            keys -= keys % 4
            kLen = keys//2
        a1 = a+keys
        a2 = a1+keys
        bSep = this.findBits(array, a1, b, kLen, cbrt)
        if bSep == -1:
            this.laziestSortExt(array, a, a2)
            this.inPlaceMerge(array, a, a2, b)
            return
        a3 = a2+bLen
        j = 1
        n = b-a3
        i: int
        binaryInsertionSort(array, a2, a3)
        bidirArrayCopy(array, a2, this.aux, 0, bLen)
        while j < cbrt:
            p = max(2, j)
            i = a3
            while i+2*j < b:
                this.mergeWithBufFWExt(array, i, i+j, i+2*j, i-p)
                i += 2*j
            if i+j < b:
                this.mergeWithBufFWExt(array, i, i+j, b, i-p)
            else:
                this.shiftFWExt(array, i-p, i, b)
            a3 -= p
            b -= p
            j *= 2
        i = b-n % (2*j)
        if i+j < b:
            this.mergeWithBufBWExt(array, i, i+j, b, b+j)
        else:
            this.shiftBWExt(array, i, b, b+j)
        i -= 2*j
        while i >= a3:
            this.mergeWithBufBWExt(array, i, i+j, i+2*j, i+3*j)
            i -= 2*j
        a3 += j
        b += j
        j *= 2
        i = a3
        while i+2*j < b:
            this.mergeWithBufFWExt(array, i, i+j, i+2*j, i-j)
            i += 2*j
        if i+j < b:
            this.mergeWithBufFWExt(array, i, i+j, b, i-j)
        else:
            this.shiftFWExt(array, i-j, i, b)
        a3 -= j
        b -= j
        j *= 2
        i = b-n % (2*j)
        if i+j < b:
            this.dualMergeBWExt(array, i, i+j, b, b+j//2)
        else:
            this.shiftBWExt(array, i, b, b+j//2)
        i -= 2*j
        while i >= a3:
            this.dualMergeBWExt(array, i, i+j, i+2*j, i+2*j+j//2)
            i -= 2*j
        a3 += j//2
        b += j//2
        j *= 2
        if keys >= j:
            rotate(array, a, a1, a3)
            a2 = a1+bLen
            if kLen >= j:
                mLvl = 2*j
                while j < kLen:
                    p = max(mLvl, j)
                    i = a3
                    while i+2*j < b:
                        this.mergeWithBufFW(array, i, i+j, i+2*j, i-p)
                        i += 2*j
                    if i+j < b:
                        this.mergeWithBufFW(array, i, i+j, b, i-p)
                    else:
                        this.shiftFW(array, i-p, i, b)
                    a3 -= p
                    b -= p
                    j *= 2
                i = b-n % (2*j)
                if i+j < b:
                    this.mergeWithBufBW(array, i, i+j, b, b+j)
                else:
                    this.shiftBW(array, i, b, b+j)
                i -= 2*j
                while i >= a3:
                    this.mergeWithBufBW(array, i, i+j, i+2*j, i+3*j)
                    i -= 2*j
                a3 += j
                b += j
                j *= 2
            if keys >= j:
                i = a3
                while i+2*j < b:
                    this.mergeWithBufFW(array, i, i+j, i+2*j, i-j)
                    i += 2*j
                if i+j < b:
                    this.mergeWithBufFW(array, i, i+j, b, i-j)
                else:
                    this.shiftFW(array, i-j, i, b)
                a3 -= j
                b -= j
                j *= 2
                i = b-n % (2*j)
                if i+j < b:
                    this.dualMergeBW(array, i, i+j, b, b+j//2)
                else:
                    this.shiftBW(array, i, b, b+j//2)
                i -= 2*j
                while i >= a3:
                    this.dualMergeBW(array, i, i+j, i+2*j, i+2*j+j//2)
                    i -= 2*j
                a3 += j//2
                b += j//2
                j *= 2
            rotate(array, a, a2, a3)
            a2 = a1+keys
            MaxHeapSort().sort(array, a, a1)
        bidirArrayCopy(this.aux, 0, array, a2, bLen)
        this.unshuffle(array, a, a1)
        limit = bLen*(kLen+2)
        k = j//bLen-1
        while j < n and min(2*j, n) <= limit:
            i = a3
            while i+2*j <= b:
                this.blockMerge(array, i, i+j, i+2*j, k, 2*k,
                                a, a+kLen, a1, a1+kLen, bLen)
                i += 2*j
            if i+j < b:
                this.blockMerge(array, i, i+j, b, k, (b-i-1) //
                                bLen-1, a, a+kLen, a1, a1+kLen, bLen)
            j *= 2
            k = 2*k+1
        while j < n:
            bLen = (2*j)//kLen
            lenA = j % bLen
            lenB = lenA
            i = a3
            while i+2*j <= b:
                this.blockMergeEasy(
                    array, i, i+j, i+2*j, lenA, lenB, kLen//2, kLen, a, a+kLen, a1, a1+kLen, bLen)
                i += 2*j
            if i+j < b:
                this.blockMergeEasy(array, i, i+j, b, lenA, (b-i-j) % bLen,
                                    kLen//2, kLen//2+(b-i-j)//bLen, a, a+kLen, a1, a1+kLen, bLen)
            j *= 2
        blockSwap(array, a1+bSep, a1+kLen+bSep, kLen-bSep)
        this.laziestSortExt(array, a, a3)
        this.redistBuffer(array, a, a3, b)


@Sort("Block Merge Sorts", "Chalice Sort", "Chalice Sort")
def chaliceSortRun(array):
    ChaliceSort(None).sort(array, 0, len(array))
