const filepreview = require('./filepreview');
const fs = require('fs');
const path = require('path');

console.log('DOCX to DOC Conversion Example\n');

// Example 1: Asynchronous conversion
console.log('=== Asynchronous Conversion ===');
function asyncExample() {
  const inputFile = './example.docx'; // Replace with your actual DOCX file path
  const outputFile = './example_converted.doc';

  // Check if input file exists (for demo purposes)
  if (!fs.existsSync(inputFile)) {
    console.log(`Input file ${inputFile} not found. Please create a sample DOCX file or update the path.`);
    console.log('You can also use a remote URL like: https://example.com/document.docx\n');
    syncExample(); // Continue with sync example
    return;
  }

  filepreview.convertDocxToDoc(inputFile, outputFile, function(error, outputPath) {
    if (error) {
      console.log('❌ Conversion failed:', error.message);
    } else {
      console.log('✅ File converted successfully to:', outputPath);
      console.log('📁 Output file size:', fs.statSync(outputPath).size, 'bytes');
    }
    console.log('');
    syncExample(); // Continue with sync example
  });
}

// Example 2: Synchronous conversion
function syncExample() {
  console.log('=== Synchronous Conversion ===');
  const inputFile = './example.docx'; // Replace with your actual DOCX file path
  const outputFile = './example_converted_sync.doc';

  try {
    // Check if input file exists (for demo purposes)
    if (!fs.existsSync(inputFile)) {
      console.log(`Input file ${inputFile} not found. Please create a sample DOCX file or update the path.`);
      console.log('You can also use a remote URL like: https://example.com/document.docx\n');
      remoteExample(); // Continue with remote example
      return;
    }

    const outputPath = filepreview.convertDocxToDocSync(inputFile, outputFile);
    console.log('✅ File converted successfully to:', outputPath);
    console.log('📁 Output file size:', fs.statSync(outputPath).size, 'bytes');
  } catch (error) {
    console.log('❌ Conversion failed:', error.message);
  }
  console.log('');
  remoteExample(); // Continue with remote example
}

// Example 3: Remote file conversion (commented out to avoid unnecessary downloads)
function remoteExample() {
  console.log('=== Remote File Conversion Example ===');
  console.log('// Example of converting a remote DOCX file:');
  console.log(`
// const remoteUrl = 'https://example.com/document.docx';
// const localOutput = './remote_converted.doc';

// filepreview.convertDocxToDoc(remoteUrl, localOutput, function(error, outputPath) {
//   if (error) {
//     console.log('❌ Remote conversion failed:', error.message);
//   } else {
//     console.log('✅ Remote file converted successfully to:', outputPath);
//   }
// });
  `);

  errorHandlingExample();
}

// Example 4: Error handling demonstration
function errorHandlingExample() {
  console.log('=== Error Handling Examples ===');
  
  // Test with wrong input format
  const wrongInputFile = './test.txt';
  const outputFile = './test_output.doc';

  filepreview.convertDocxToDoc(wrongInputFile, outputFile, function(error, outputPath) {
    if (error) {
      console.log('✅ Expected error for wrong input format:', error.message);
    } else {
      console.log('❌ This should not succeed');
    }

    // Test with wrong output format
    const docxInput = './example.docx';
    const wrongOutput = './output.pdf';

    filepreview.convertDocxToDoc(docxInput, wrongOutput, function(error, outputPath) {
      if (error) {
        console.log('✅ Expected error for wrong output format:', error.message);
      } else {
        console.log('❌ This should not succeed');
      }
      
      console.log('\n=== Demo Complete ===');
      console.log('To test with actual files:');
      console.log('1. Create or place a DOCX file as "example.docx" in this directory');
      console.log('2. Run this script again with: node example_docx_to_doc.js');
      console.log('3. Check the generated DOC files');
    });
  });
}

// Start the examples
asyncExample();