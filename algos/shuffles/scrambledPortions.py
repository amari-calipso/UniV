import math
import random

@Shuffle("Scrambled Tail")
def scrambledTailShuffle(array):
    aux = sortingVisualizer.createValueArray(len(array))
    i = 0
    j = 0
    k = 0
    while i < len(array):
        if random.random() < 1.0/7.0:
            aux[k].write(array[i])
            k += 1
        else:
            array[j].write(array[i])
            j += 1
        i += 1
    arrayCopy(aux, 0, array, j, k)
    shuffleRandom(array, j, len(array))


@Shuffle("Scrambled Head")
def scrambledHeadShuffle(array):
    aux = sortingVisualizer.createValueArray(len(array))
    i = len(array)-1
    j = i
    k = 0
    while i >= 0:
        if random.random() < 1.0/7.0:
            aux[k].write(array[i])
            k += 1
        else:
            array[j].write(array[i])
            j -= 1
        i -= 1
    arrayCopy(aux, 0, array, 0, k)
    shuffleRandom(array, 0, j)


@Shuffle("Noisy")
def noisyShuffle(array):
    size = max(4, int(math.sqrt(len(array))//2))
    i = 0
    while i+size <= len(array):
        shuffleRandom(array, i, i+size)
        i += random.randint(1, size)
    shuffleRandom(array, i, len(array))
