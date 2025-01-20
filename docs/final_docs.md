## ZPR 24Z - Narzędzie CLI do optymalizacji plików SVG

### Skład zespołu
- Jakub Proboszcz
- Paweł Kochański

## Oryginalny temat projektu

Celem projektu jest stworzenie aplikacji, która pozwala na optymalizację pliku `.svg` pod kątem rozmiaru. Referencyjnym przykładem takiej aplikacji jest [svgo](https://github.com/svg/svgo). Aplikacja powinna działać z poziomu konsoli (CLI).

## Opis architektury

![diagram uml](uml.png "Diagram UML")

## Zaimplementowana funkcjonalność

W ramach projektu zaimplementowaliśmy następujący podzbiór optymalizacji:
- zmiana nazw `id` na minimalnej długości, usuwanie niepotrzebnych
- usuwanie znaków nowej linii i nadmiarowych spacji z atrybutów
- zaokrąglanie liczb zmiennoprzecinkowych do określonej precyzji
- redukowanie niepotrzebnych grup
- konwertowanie elips będących kołami w koła
- łączenie transformacji w pojedynczą macierz
- łączenie ścieżek
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
- usuwanie nieużywanych definicji
- usuwanie bezużytecznych atrybutów `stroke` i `fill`
- zamiana identycznych ścieżek na `<use>` tej samej ścieżki
- sortowanie atrybutów dla lepszej kompresji

## Niewykonane optymalizacje

- usuwanie `enable-background` w przypadku gdy odpowiada wymiarami wymiarom w tagu `svg`

Atrybut `enable-background` nie jest wspierany w obecnym standardzie SVG.

- łączenie transformacji w pojedynczą macierz

- aplikowanie transformacji na ścieżki
- usuwanie ścieżek rysowanych poza ekranem

Struktura ścieżek w SVG jest dość skomplikowana, w związku z czym zabrakło nam czasu na zaimplementowanie tej optymalizacji

- usuwanie nieużywanych przestrzeni nazw

Funkcjonalność ta jest w większości pokryta przez "usuwanie przestrzeni nazw, elementów i atrybutów edytorów", jako że jedyną przestrzenią nazw pozostałą po wyżej wymienionej optymalizacji pozostaje "xlink".

- konwertowanie jednopunktowych gradientów w czysty kolor
- łączenie styli

Parsowanie CSS'a jest zadaniem nietrywialnym i z racji tego woleliśmy skupić się na optymalizacjach związanych bezpośrednio z SVG.