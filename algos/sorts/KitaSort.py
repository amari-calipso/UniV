class KitaSort:
    def __init__(this):
        this.buf = None
        this.tags = None
        this.tTmp = None

    def mergeTo(this, from_, to, a, m, b, p):
        i = a
        j = m
        while i < m and j < b:
            if from_[i] <= from_[j]:
                to[p].write(from_[i])
                i += 1
            else:
                to[p].write(from_[j])
                j += 1
            p += 1
        while i < m:
            to[p].write(from_[i])
            p += 1
            i += 1
        while j < b:
            to[p].write(from_[j])
            p += 1
            j += 1

    def pingPongMerge(this, array, buf, a, m1, m2, m3, b):
        p = 0
        p1 = p+m2-a
        pEnd = p+b-a
        this.mergeTo(array, buf, a, m1, m2, p)
        this.mergeTo(array, buf, m2, m3, b, p1)
        this.mergeTo(buf, array, p, p1, pEnd, a)

    def mergeBWExt(this, array, tmp, a, m, b):
        s = b-m
        bidirArrayCopy(array, m, tmp, 0, s)
        i = s-1
        j = m-1
        b -= 1
        while i >= 0 and j >= a:
            if tmp[i] >= array[j]:
                array[b].write(tmp[i])
                i -= 1
            else:
                array[b].write(array[j])
                j -= 1
            b -= 1
        b -= 1
        while i >= 0:
            array[b].write(tmp[i])
            b -= 1
            i -= 1

    def blockMerge(this, array, a, m, b, bLen):
        ta = a//bLen
        tm = m//bLen
        tb = b//bLen
        ti = ta
        tj = tm
        i = a+this.tags[ti]*bLen
        j = m+this.tags[tj]*bLen
        c = 0
        ci = 0
        cj = 0
        bi = ti
        bj = tj
        l = 0
        r = 0
        t = 2
        p: int
        lLeft = True
        rLeft = True
        lBuf: bool
        for k in range(2*bLen):
            if lLeft and ((not rLeft) or array[i] <= array[j]):
                this.buf[k].write(array[i])
                i += 1
                l += 1
                ci += 1
                if ci == bLen:
                    ti += 1
                    if ti == tm:
                        lLeft = False
                    else:
                        i = a+this.tags[ti]*bLen
                        ci = 0
            else:
                this.buf[k].write(array[j])
                j += 1
                r += 1
                cj += 1
                if cj == bLen:
                    tj += 1
                    if tj == tb:
                        rLeft = False
                    else:
                        j = m+this.tags[tj]*bLen
                        cj = 0
        lBuf = l >= r
        if lBuf:
            p = a+this.tags[bi]*bLen
        else:
            p = m+this.tags[bj]*bLen
        while True:
            if lLeft and ((not rLeft) or array[i] <= array[j]):
                array[p].write(array[i])
                p += 1
                i += 1
                l += 1
                ci += 1
                if ci == bLen:
                    ti += 1
                    if ti == tm:
                        lLeft = False
                    else:
                        i = a+this.tags[ti]*bLen
                        ci = 0
            else:
                array[p].write(array[j])
                p += 1
                j += 1
                r += 1
                cj += 1
                if cj == bLen:
                    tj += 1
                    if tj == tb:
                        rLeft = False
                    else:
                        j = m+this.tags[tj]*bLen
                        cj = 0
            c += 1
            if c == bLen:
                if lBuf:
                    l -= bLen
                    this.tTmp[t].write(this.tags[bi])
                    t += 1
                    bi += 1
                else:
                    r -= bLen
                    this.tTmp[t].write(this.tags[bj]+tm-ta)
                    t += 1
                    bj += 1
                lBuf = l >= r
                p = a+this.tags[bi]*bLen if lBuf else m + \
                    this.tags[bj]*bLen
                c = 0
            if not (lLeft or rLeft):
                break
        p = 0
        t = 0
        while l > 0:
            bidirArrayCopy(this.buf, p, array, a +
                           this.tags[bi]*bLen, bLen)
            this.tTmp[t].write(this.tags[bi])
            t += 1
            bi += 1
            p += bLen
            l -= bLen
        while r > 0:
            bidirArrayCopy(this.buf, p, array, m +
                           this.tags[bj]*bLen, bLen)
            this.tTmp[t].write(this.tags[bj]+tm-ta)
            t += 1
            bj += 1
            p += bLen
            r -= bLen
        bidirArrayCopy(this.tTmp, 0, this.tags, ta, tb-ta)

    def blockCycle(this, array, a, bLen, bCnt):
        for i in range(bCnt):
            if this.tags[i] != i:
                bidirArrayCopy(array, a+i*bLen, this.buf, 0, bLen)
                j = i
                next = this.tags[i]
                while True:
                    bidirArrayCopy(array, a+next*bLen, array, a+j*bLen, bLen)
                    this.tags[j].write(j)
                    j = next
                    next = this.tags[next]
                    if not (next != i):
                        break
                bidirArrayCopy(this.buf, 0, array, a+j*bLen, bLen)
                this.tags[j].write(j)

    def sort(this, array, a, b):
        length = b-a
        if length <= 32:
            binaryInsertionSort(array, a, b)
            return
        sqrtLg = (32-javaNumberOfLeadingZeros(length-1))//2
        bLen = 1 << sqrtLg
        tLen = length//bLen
        bufLen = 2*bLen
        this.buf = sortingVisualizer.createValueArray(bufLen)
        this.tags = sortingVisualizer.createValueArray(tLen)
        this.tTmp = sortingVisualizer.createValueArray(tLen)
        sortingVisualizer.setNonOrigAux([this.tags, this.tTmp])
        b1 = b-length % bLen
        j = 1
        if sqrtLg % 2 == 0:
            i = a+1
            while i < b1:
                if array[i-1] > array[i]:
                    array[i-1].swap(array[i])
                i += 2
            j *= 2
        while j < bufLen:
            i = a
            while i+j < b1:
                this.pingPongMerge(array, this.buf, i, i+j,
                                   min(i+2*j, b1), min(i+3*j, b1), min(i+4*j, b1))
                i += 4*j
            j *= 4
        for i in range(tLen):
            this.tags[i].write(i & 1)
        while j < length:
            i = a
            while i+j < b1:
                this.blockMerge(array, i, i+j, min(i+2*j, b1), bLen)
                i += 2*j
            j *= 2
        this.blockCycle(array, a, bLen, tLen)
        if b1 < b:
            binaryInsertionSort(array, b1, b)
            this.mergeBWExt(array, this.buf, a, b1, b)


@Sort("Block Merge Sorts", "Kita Sort", "Kita Sort")
def kitaSortRun(array):
    KitaSort().sort(array, 0, len(array))
