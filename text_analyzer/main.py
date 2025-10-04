file_name = input("Введите имя файла: ")

try:
    with open(file_name, 'r', encoding='utf-8') as f:
        lines = f.readlines()
except FileNotFoundError:
    print(f"Ошибка: файл '{file_name}' не найден.")
    exit()

print("1: Количество строк")
print("2: Количество символов")
print("3: Количество пустых строк")
print("4: Частоту символов")
print("Например: 1,4 или all для всего")

choice = input("Ваш выбор: ").strip()

if 'all' in choice or '1' in choice:
    print(f"Всего строк: {len(lines)}")

if 'all' in choice or '2' in choice:
    full_text = "".join(lines)
    print(f"Всего символов: {len(full_text)}")

if 'all' in choice or '3' in choice:
    empty_count = 0
    for line in lines:
        if line.strip() == "":
            empty_count += 1
    print(f"Пустых строк: {empty_count}")

if 'all' in choice or '4' in choice:
    char_counts = {}
    full_text = "".join(lines)

    for char in full_text:
        if char == '\n':
            continue
        
        if char in char_counts:
            char_counts[char] += 1
        else:
            char_counts[char] = 1
            
    print("Частота символов:")
    for char, count in sorted(char_counts.items()):
        print(f"  '{char}': {count}")