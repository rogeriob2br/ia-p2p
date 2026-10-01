#!/usr/bin/env python3
"""
Script para extrair texto de PDFs e converter para Markdown
"""
import os
import sys
from pathlib import Path

try:
    import pymupdf  # PyMuPDF
    HAS_PYMUPDF = True
except ImportError:
    HAS_PYMUPDF = False
    print("PyMuPDF não encontrado. Tentando pdfplumber...")
    
try:
    import pdfplumber
    HAS_PDFPLUMBER = True
except ImportError:
    HAS_PDFPLUMBER = False
    print("pdfplumber não encontrado.")

if not HAS_PYMUPDF and not HAS_PDFPLUMBER:
    print("\nERRO: Nenhuma biblioteca de PDF encontrada!")
    print("Instale uma delas:")
    print("  pip install pymupdf")
    print("  pip install pdfplumber")
    sys.exit(1)

def extract_with_pymupdf(pdf_path):
    """Extrai texto usando PyMuPDF"""
    doc = pymupdf.open(pdf_path)
    text = ""
    for page_num, page in enumerate(doc, 1):
        text += f"\n\n## Página {page_num}\n\n"
        text += page.get_text()
    doc.close()
    return text

def extract_with_pdfplumber(pdf_path):
    """Extrai texto usando pdfplumber"""
    text = ""
    with pdfplumber.open(pdf_path) as pdf:
        for page_num, page in enumerate(pdf.pages, 1):
            text += f"\n\n## Página {page_num}\n\n"
            page_text = page.extract_text()
            if page_text:
                text += page_text
    return text

def pdf_to_markdown(pdf_path, output_dir):
    """Converte PDF para Markdown"""
    pdf_path = Path(pdf_path)
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Processando: {pdf_path.name}")
    
    # Extrai texto
    try:
        if HAS_PYMUPDF:
            text = extract_with_pymupdf(pdf_path)
        else:
            text = extract_with_pdfplumber(pdf_path)
    except Exception as e:
        print(f"  ERRO ao extrair {pdf_path.name}: {e}")
        return False
    
    # Cria arquivo markdown
    md_filename = pdf_path.stem + ".md"
    md_path = output_dir / md_filename
    
    # Adiciona cabeçalho
    markdown_content = f"# {pdf_path.stem}\n\n"
    markdown_content += f"*Extraído de: {pdf_path.name}*\n\n"
    markdown_content += "---\n\n"
    markdown_content += text
    
    # Salva
    with open(md_path, 'w', encoding='utf-8') as f:
        f.write(markdown_content)
    
    print(f"  ✓ Salvo em: {md_path}")
    return True

def main():
    # Diretório dos PDFs
    pdf_dir = Path("docs/artigos")
    output_dir = Path("docs/artigos_md")
    
    if not pdf_dir.exists():
        print(f"ERRO: Diretório {pdf_dir} não encontrado!")
        sys.exit(1)
    
    # Lista todos os PDFs
    pdf_files = list(pdf_dir.glob("*.pdf"))
    
    if not pdf_files:
        print(f"Nenhum PDF encontrado em {pdf_dir}")
        sys.exit(1)
    
    print(f"\nEncontrados {len(pdf_files)} PDFs\n")
    print(f"Biblioteca usada: {'PyMuPDF' if HAS_PYMUPDF else 'pdfplumber'}\n")
    
    # Processa cada PDF
    success_count = 0
    for pdf_file in pdf_files:
        if pdf_to_markdown(pdf_file, output_dir):
            success_count += 1
    
    print(f"\n{'='*60}")
    print(f"Concluído: {success_count}/{len(pdf_files)} PDFs convertidos")
    print(f"Arquivos salvos em: {output_dir}")
    print(f"{'='*60}\n")

if __name__ == "__main__":
    main()
