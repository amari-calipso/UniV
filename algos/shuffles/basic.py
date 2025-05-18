@Shuffle("Few random")
def fewRandomShuffle(array):
    for _ in range(max(len(array)//20, 1)):
        array[random.randint(0, len(array)-1)
              ].swap(array[random.randint(0, len(array)-1)])


@Shuffle("Final Merge Pass")
def finalMergeShuffle(array):
    count = 2
    temp = sortingVisualizer.createValueArray(len(array))
    k = 0
    j = 0
    while j < count:
        i = j
        while i < len(array):
            temp[k].write(array[i])
            i += count
            k += 1
        j += 1
    arrayCopy(temp, 0, array, 0, len(array))
