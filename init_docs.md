## ZPR 24Z - Narzędzie CLI do optymalizacji plików SVG

### Skład zespołu
- Jakub Proboszcz
- Paweł Kochański

## Oryginalny temat projektu

Celem projektu jest stworzenie aplikacji, która pozwala na optymalizację pliku `.svg` pod kątem rozmiaru. Referencyjnym przykładem takiej aplikacji jest [svgo](https://github.com/svg/svgo). Aplikacja powinna działać z poziomu konsoli (CLI).

## Technologia

Aplikacja zostanie napisana w Rust; do zarządzania zależnościami i testów użyjemy `cargo`. \
Do zarządzania projektem użyjemy [just](https://github.com/casey/just).

## Podział na podproblemy

### Interfejs użytkownika

CLI aplikacji będzie udostępniał opcje:
- `--help` zawierający instrukcję obsługi aplikacji
- przekazywanie listy plików wejściowych i wyjściowych
- ustalanie, które optymalizacje mają zostać użyte
- jeżeli pewna grupa optymalizacji będzie szczególnie powolna, opcje włączenia lub wyłączenia konkretnych grup optymalizacji (analogicznie do opcji `-O1`, `-O2` obecnych w kompilatorach)

Interfejs będzie napisany z użyciem biblioteki [clap](https://crates.io/crates/clap).

### Parsowanie formatu SVG

Program będzie parsował plik wejściowy do drzewa składniowego, nakładał optymalizacje na drzewo składniowe, następnie zamieniał wynikowe drzewo składniowe z powrotem na format SVG.

Parsowanie pliku `.svg` do drzewa składniowego oraz proces odwrotny będą wykonane z użyciem biblioteki [svg](https://crates.io/crates/svg).

### Optymalizacja drzewa składniowego SVG

W ramach projektu zaimplementujemy podzbiór optymalizacji wbudowanych w program [svgo](https://github.com/svg/svgo).\
W szczególności wykonamy:
- zmiana nazw `id` na minimalnej długości, usuwanie niepotrzebnych
- usuwanie znaków nowej linii i nadmiarowych spacji z atrybutów
- usuwanie `enable-background` w przypadku gdy odpowiada wymiarami wymiarom w tagu `svg`
- zaokrąglanie liczb zmiennoprzecinkowych do określonej precyzji
- redukowanie niepotrzebnych grup
- konwertowanie elips będących kołami w koła
- konwertowanie jednopunktowych gradientów w czysty kolor
- łączenie transformacji w pojedynczą macierz
- aplikowanie transformacji na ścieżki
- łączenie ścieżek
- łączenie styli
- wyłączanie wspólnych atrybutów elementów grupy do atrybutów grupy
- usuwanie komentarzy
- usuwanie deklaracji `DOCTYPE`
- usuwanie elementów: `<desc>`, `<metadata>`, `<title>`, `<xml>`
- usuwanie przestrzeni nazw, elementów i atrybutów edytorów
- usuwanie pustych atrybutów
- usuwanie pustych kontenerów
- usuwanie pustych elementów `<text>`, `<tspan>`, `<tref>`
- zamiana atrybutów `width` i `height` na `viewBox`
- usuwanie niewidocznych elementów
- usuwanie ścieżek rysowanych poza ekranem
- usuwanie nieużywanych przestrzeni nazw
- usuwanie nieużywanych definicji
- usuwanie bezużytecznych atrybutów `stroke` i `fill`
- zamiana identycznych ścieżek na `<use>` tej samej ścieżki
- sortowanie atrybutów dla lepszej kompresji
