/*

  filepreview : A file preview generator for node.js

*/

var child_process = require('child_process');
var crypto = require('crypto');
var path = require('path');
var fs = require('fs');
var os = require('os');
var mimedb = require('./db.json');
var download = require('download-file')

module.exports = {
  generate: function(input_original, output, options, callback) {
    // Normalize arguments

    var input = input_original;

    if (typeof options === 'function') {
      callback = options;
      options = {};
    } else {
      options = options || {};
    }

    // Check for supported output format
    var extOutput = path.extname(output).toLowerCase().replace('.','');
    var extInput = path.extname(input).toLowerCase().replace('.','');

    if (
      extOutput != 'gif' &&
      extOutput != 'jpg' &&
      extOutput != 'png'
    ) {
      return callback(true);
    }

    var fileType = 'other';

    root:
    for ( var index in mimedb ) {
      if ( 'extensions' in mimedb[index] ) {
        for ( var indexExt in mimedb[index].extensions ) {
          if ( mimedb[index].extensions[indexExt] == extInput ) {
            if ( index.split('/')[0] == 'image' ) {
              fileType = 'image';
            } else if ( index.split('/')[0] == 'video' ) {
              fileType = 'video';
            } else {
              fileType = 'other';
            }

            break root;
          }
        }
      }
    }

    if ( extInput == 'pdf' ) {
      fileType = 'image';
    }

    if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
      var url = input.split("/");
      var url_filename = url[url.length - 1];
      var hash = crypto.createHash('sha512');
      hash.update(Math.random().toString());
      hash = hash.digest('hex');
      var temp_input = path.join(os.tmpdir(), hash + url_filename);
      curlArgs = ['--silent', '-L', input, '-o', temp_input];
      child_process.execFileSync("curl", curlArgs);
      input = temp_input;
    }

    fs.lstat(input, function(error, stats) {
      if (error) return callback(error);
      if (!stats.isFile()) {
        return callback(true);
      } else {
        if ( fileType == 'video' ) {
          var ffmpegArgs = ['-y', '-i', input, '-vf', 'thumbnail', '-frames:v', '1'];
          if (options.width > 0 && options.height > 0) {
            ffmpegArgs.splice(4, 1, 'thumbnail,scale=' + options.width + ':' + options.height);
          }
          if (options.hasOwnProperty('previewTime')) {
            ffmpegArgs.push("-ss");
            ffmpegArgs.push(options.previewTime)
          }
          ffmpegArgs.push(output);

          child_process.execFile('ffmpeg', ffmpegArgs, function(error) {
            if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
              fs.unlinkSync(input);
            }

            if (error) return callback(error);
            return callback();
          });
        }

        if ( fileType == 'image' ) {
          var convertArgs = [input + '[0]', output];
          if (options.width > 0 && options.height > 0) {
            convertArgs.splice(0, 0, '-resize', options.width + 'x' + options.height);
          }
          if (options.quality) {
            convertArgs.splice(0, 0, '-quality', options.quality);
          }
          child_process.execFile('convert', convertArgs, function(error) {
            if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
              fs.unlinkSync(input);
            }
            if (error) return callback(error);
            return callback();
          });
        }

        if ( fileType == 'other' ) {
          var hash = crypto.createHash('sha512');
          hash.update(Math.random().toString());
          hash = hash.digest('hex');

          var tempPDF = path.join(os.tmpdir(), hash + '.pdf');

          child_process.execFile('unoconv', ['-e', 'PageRange=1', '-o', tempPDF, input], function(error) {
            if (error) return callback(error);
            var convertOtherArgs = [tempPDF + '[0]', output];
            if (options.width > 0 && options.height > 0) {
              convertOtherArgs.splice(0, 0, '-resize', options.width + 'x' + options.height);
            }
            if (options.quality) {
              convertOtherArgs.splice(0, 0, '-quality', options.quality);
            }
            child_process.execFile('convert', convertOtherArgs, function(error) {
              if (error) return callback(error);
              fs.unlink(tempPDF, function(error) {
                if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
                  fs.unlinkSync(input);
                }
                if (error) return callback(error);
                return callback();
              });
            });
          });
        }
      }
    });
  },

  generateSync: function(input_original, output, options) {

    options = options || {};

    var input = input_original;

    // Check for supported output format
    var extOutput = path.extname(output).toLowerCase().replace('.','');
    var extInput = path.extname(input).toLowerCase().replace('.','');

    if (
      extOutput != 'gif' &&
      extOutput != 'jpg' &&
      extOutput != 'png'
    ) {
      return false;
    }

    var fileType = 'other';

    root:
    for ( var index in mimedb ) {
      if ( 'extensions' in mimedb[index] ) {
        for ( var indexExt in mimedb[index].extensions ) {
          if ( mimedb[index].extensions[indexExt] == extInput ) {
            if ( index.split('/')[0] == 'image' ) {
              fileType = 'image';
            } else if ( index.split('/')[0] == 'video' ) {
              fileType = 'video';
            } else {
              fileType = 'other';
            }

            break root;
          }
        }
      }
    }

    if ( extInput == 'pdf' ) {
      fileType = 'image';
    }

    if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
      var url = input.split("/");
      var url_filename = url[url.length - 1];
      var hash = crypto.createHash('sha512');
      hash.update(Math.random().toString());
      hash = hash.digest('hex');
      var temp_input = path.join(os.tmpdir(), hash + url_filename);
      curlArgs = ['--silent', '-L', input, '-o', temp_input];
      child_process.execFileSync("curl", curlArgs);
      input = temp_input;
    }

    try {
        stats = fs.lstatSync(input);

        if (!stats.isFile()) {
          return false;
        }
    } catch (e) {
        return false;
    }

    if ( fileType == 'video' ) {
      try {
        var ffmpegArgs = ['-y', '-i', input, '-vf', 'thumbnail', '-frames:v', '1', output];
        if (options.width > 0 && options.height > 0) {
          ffmpegArgs.splice(4, 1, 'thumbnail,scale=' + options.width + ':' + options.height)
        }
        child_process.execFileSync('ffmpeg', ffmpegArgs);
        if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
          fs.unlinkSync(input);
        }
        return true;
      } catch (e) {
        return false;
      }
    }

    if ( fileType == 'image' ) {
      try {
        var convertArgs = [input + '[0]', output];
        if (options.width > 0 && options.height > 0) {
          convertArgs.splice(0, 0, '-resize', options.width + 'x' + options.height);
        }
        if (options.quality) {
          convertArgs.splice(0, 0, '-quality', options.quality);
        }
        child_process.execFileSync('convert', convertArgs);
        if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
          fs.unlinkSync(input);
        }
        return true;
      } catch (e) {
        return false;
      }
    }

    if ( fileType == 'other' ) {
      try {
        var hash = crypto.createHash('sha512');
        hash.update(Math.random().toString());
        hash = hash.digest('hex');

        var tempPDF = path.join(os.tmpdir(), hash + '.pdf');

        child_process.execFileSync('unoconv', ['-e', 'PageRange=1', '-o', tempPDF, input]);

        var convertOtherArgs = [tempPDF + '[0]', output];
        if (options.width > 0 && options.height > 0) {
          convertOtherArgs.splice(0, 0, '-resize', options.width + 'x' + options.height);
        }
        if (options.quality) {
          convertOtherArgs.splice(0, 0, '-quality', options.quality);
        }
        child_process.execFileSync('convert', convertOtherArgs);
        fs.unlinkSync(tempPDF);
        if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
          fs.unlinkSync(input);
        }
        return true;
      } catch (e) {
        return false;
      }
    }
  },

  convertDocxToDoc: function(input_original, output, callback) {
    var input = input_original;

    if (typeof callback !== 'function') {
      throw new Error('Callback function is required');
    }

    // Check for supported input and output formats
    var extOutput = path.extname(output).toLowerCase().replace('.','');
    var extInput = path.extname(input).toLowerCase().replace('.','');

    if (extInput !== 'docx') {
      return callback(new Error('Input file must be a DOCX file'));
    }

    if (extOutput !== 'doc') {
      return callback(new Error('Output file must have .doc extension'));
    }

    // Handle remote files
    if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
      var url = input.split("/");
      var url_filename = url[url.length - 1];
      var hash = crypto.createHash('sha512');
      hash.update(Math.random().toString());
      hash = hash.digest('hex');
      var temp_input = path.join(os.tmpdir(), hash + url_filename);
      curlArgs = ['--silent', '-L', input, '-o', temp_input];
      
      try {
        child_process.execFileSync("curl", curlArgs);
        input = temp_input;
      } catch (error) {
        return callback(new Error('Failed to download remote file: ' + error.message));
      }
    }

    // Verify input file exists
    fs.lstat(input, function(error, stats) {
      if (error) return callback(new Error('Input file not found: ' + error.message));
      if (!stats.isFile()) {
        return callback(new Error('Input path is not a file'));
      }

      // Convert DOCX to DOC using unoconv
      var unoconvArgs = ['-f', 'doc', '-o', output, input];
      
      child_process.execFile('unoconv', unoconvArgs, function(error, stdout, stderr) {
        // Clean up temporary file if it was downloaded
        if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
          try {
            fs.unlinkSync(input);
          } catch (unlinkError) {
            // Ignore cleanup errors
          }
        }

        if (error) {
          return callback(new Error('DOCX to DOC conversion failed: ' + (stderr || error.message)));
        }

        // Verify output file was created
        fs.access(output, fs.constants.F_OK, function(accessError) {
          if (accessError) {
            return callback(new Error('Output file was not created'));
          }
          return callback(null, output);
        });
      });
    });
  },

  convertDocxToDocSync: function(input_original, output) {
    var input = input_original;

    // Check for supported input and output formats
    var extOutput = path.extname(output).toLowerCase().replace('.','');
    var extInput = path.extname(input).toLowerCase().replace('.','');

    if (extInput !== 'docx') {
      throw new Error('Input file must be a DOCX file');
    }

    if (extOutput !== 'doc') {
      throw new Error('Output file must have .doc extension');
    }

    // Handle remote files
    if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
      var url = input.split("/");
      var url_filename = url[url.length - 1];
      var hash = crypto.createHash('sha512');
      hash.update(Math.random().toString());
      hash = hash.digest('hex');
      var temp_input = path.join(os.tmpdir(), hash + url_filename);
      curlArgs = ['--silent', '-L', input, '-o', temp_input];
      
      try {
        child_process.execFileSync("curl", curlArgs);
        input = temp_input;
      } catch (error) {
        throw new Error('Failed to download remote file: ' + error.message);
      }
    }

    try {
      // Verify input file exists
      var stats = fs.lstatSync(input);
      if (!stats.isFile()) {
        throw new Error('Input path is not a file');
      }

      // Convert DOCX to DOC using unoconv
      var unoconvArgs = ['-f', 'doc', '-o', output, input];
      child_process.execFileSync('unoconv', unoconvArgs);

      // Clean up temporary file if it was downloaded
      if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
        try {
          fs.unlinkSync(input);
        } catch (unlinkError) {
          // Ignore cleanup errors
        }
      }

      // Verify output file was created
      try {
        fs.accessSync(output, fs.constants.F_OK);
        return output;
      } catch (accessError) {
        throw new Error('Output file was not created');
      }

    } catch (error) {
      // Clean up temporary file if it was downloaded
      if (input_original.indexOf("http://") == 0 || input_original.indexOf("https://") == 0) {
        try {
          fs.unlinkSync(input);
        } catch (unlinkError) {
          // Ignore cleanup errors
        }
      }
      
      if (error.message.includes('Input file not found') || 
          error.message.includes('Input path is not a file') ||
          error.message.includes('Input file must be') ||
          error.message.includes('Output file must have') ||
          error.message.includes('Failed to download') ||
          error.message.includes('Output file was not created')) {
        throw error;
      } else {
        throw new Error('DOCX to DOC conversion failed: ' + error.message);
      }
    }
  }
};
