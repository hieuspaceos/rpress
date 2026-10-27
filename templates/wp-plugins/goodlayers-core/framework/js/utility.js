(function($){
	"use strict";

	// count recursive array/object
	window.gdlr_core_array_count = function(arr){
        var count = 0;
        for(var k in arr){
            count++;
            if( typeof(arr[k]) == 'object' ){
                count += window.gdlr_core_array_count(arr[k]);
            }
        }
        return count;
    } // gdlr_core_array_count

	// create the alert message
	window.gdlr_core_alert_box = function(options){
	
        var settings = $.extend({
			status: '',
			head: '',
			message: '',
			duration: 1500
        }, options);

		if( settings.status == 'success' ){
			settings.icon = 'fa fa-check';
		}else if( settings.status == 'failed' ){
			settings.icon = 'fa fa-remove';
		}
		
		var alert_box = $('<div class="gdlr-core-alert-box-wrapper">\
				<div class="gdlr-core-alert-box-head">\
					<span class="gdlr-core-alert-box-icon ' + settings.icon + '"></span>\
					<span class="gdlr-core-alert-box-head">' + settings.head + '</span>\
				</div>' +
				((settings.message.length > 0)? '<div class="gdlr-core-alert-box-text">' + settings.message + '</div>': '') +
			'</div>').appendTo($('body'));
		
		alert_box.css({opacity: 0}).animate({opacity:1}, 150);
		
		// center the alert box position
		alert_box.css({
			'margin-left': -(alert_box.outerWidth() / 2),
			'margin-top': -(alert_box.outerHeight() / 2)
		});
				
		// animate the alert box
		alert_box.animate({opacity:1}, function(){
			$(this).delay(settings.duration).fadeOut(200, function(){
				$(this).remove();
			});
		});
		
	} // gdlr_core_alert_box
	
	// create the conformation message
	window.gdlr_core_confirm_box = function(options){

        var settings = $.extend({
			head: gdlr_utility.confirm_head,
			text: gdlr_utility.confirm_text,
			sub: gdlr_utility.confirm_sub,
			success:  function(){}
        }, options);
		
		var confirm_overlay = $('<div class="gdlr-conform-box-overlay"></div>').appendTo($('body'));
		var confirm_button = $('<span class="gdlr-core-confirm-box-button gdlr-core-yes">' + gdlr_utility.confirm_yes + '</span>');
		var decline_button = $('<span class="gdlr-core-confirm-box-button gdlr-core-no">' + gdlr_utility.confirm_no + '</span>');
		
		var confirm_box = $('<div class="gdlr-core-confirm-box-wrapper">\
				<div class="gdlr-core-confirm-box-head">' + settings.head + '</div>\
				<div class="gdlr-core-confirm-box-content-wrapper" >\
					<div class="gdlr-core-confirm-box-text">' + settings.text + '</div>\
					<div class="gdlr-core-confirm-box-sub">' + settings.sub + '</div>\
				</div>\
			</div>').insertAfter(confirm_overlay);
	
	
		$('<div class="gdlr-core-confirm-box-button-wrapper"></div>')
			.append(decline_button).append(confirm_button)
			.appendTo(confirm_box);
		
		// center the alert box position
		confirm_box.css({
			'margin-left': -(confirm_box.outerWidth() / 2),
			'margin-top': -(confirm_box.outerHeight() / 2)
		});
				
		// animate the alert box
		confirm_overlay.css({opacity: 0}).animate({opacity:0.6}, 200);
		confirm_box.css({opacity: 0}).animate({opacity:1}, 200);
		
		confirm_button.click(function(){
			if(typeof(settings.success) == 'function'){ 
				settings.success();
			}
			confirm_overlay.fadeOut(200, function(){
				$(this).remove();
			});
			confirm_box.fadeOut(200, function(){
				$(this).remove();
			});
		});
		decline_button.click(function(){
			confirm_overlay.fadeOut(200, function(){
				$(this).remove();
			});
			confirm_box.fadeOut(200, function(){
				$(this).remove();
			});
		});
		
	} // gdlr_core_confirm_box
	
	// ref : http://unscriptable.com/2009/03/20/debouncing-javascript-methods/
	// ensure 1 is fired
	window.gdlr_core_debounce = function(func, threshold, execAsap){
		
		var timeout;

		return function debounced(){
			
			var obj = this, args = arguments;
			
			function delayed(){
				if( !execAsap ){
					func.apply(obj, args);
				}
				timeout = null;
			};

			if( timeout ){
				clearTimeout(timeout);
			}else if( execAsap ){
				func.apply(obj, args);
			}
			timeout = setTimeout(delayed, threshold);
		};
	}	
	
	// reduce the event occurance
	window.gdlr_core_throttling = function(func, threshold){
		
		var timeout;

		return function throttled(){
			var obj = this, args = arguments;
			
			function delayed(){
				func.apply(obj, args);
				timeout = null;
			};

			if( !timeout ){
				timeout = setTimeout(delayed, threshold);
			}
		};
	}

	// download file
	window.gdlr_core_download_content = function(data, filename){
		var element = document.createElement('a');
		element.setAttribute('href', 'data:application/json;charset=utf-8,' + encodeURIComponent(data));
		element.setAttribute('download', filename);

		element.style.display = 'none';
		document.body.appendChild(element);

		element.click();

		document.body.removeChild(element);
	}
	window.gdlr_core_download_file = function(url, filename){
		var element = document.createElement('a');
		element.setAttribute('href', url);
		element.setAttribute('download', filename);

		element.style.display = 'none';
		document.body.appendChild(element);

		element.click();

		document.body.removeChild(element);
	}

	// clone without event 
	// also remove jquery ui-sortable data
	$.fn.gdlr_core_clone = function(){
		var clone = $(this).clone(true).off().removeData('uiSortable sortableItem').removeClass('ui-sortable ui-sortable-handle');
		clone.find('*').off().removeData('uiSortable sortableItem').removeClass('ui-sortable ui-sortable-handle');

		return clone;
	}

	// from wp-admin/js/editor.js
	// Replace paragraphs with double line breaks
	window.gdlr_core_removep = function( html ) {
		var blocklist = 'blockquote|ul|ol|li|dl|dt|dd|table|thead|tbody|tfoot|tr|th|td|h[1-6]|fieldset',
			blocklist1 = blocklist + '|div|p',
			blocklist2 = blocklist + '|pre',
			preserve_linebreaks = false,
			preserve_br = false;

		if ( ! html ) {
			return '';
		}

		// Protect pre|script tags
		if ( html.indexOf( '<pre' ) !== -1 || html.indexOf( '<script' ) !== -1 ) {
			preserve_linebreaks = true;
			html = html.replace( /<(pre|script)[^>]*>[\s\S]+?<\/\1>/g, function( a ) {
				a = a.replace( /<br ?\/?>(\r\n|\n)?/g, '<wp-line-break>' );
				a = a.replace( /<\/?p( [^>]*)?>(\r\n|\n)?/g, '<wp-line-break>' );
				return a.replace( /\r?\n/g, '<wp-line-break>' );
			});
		}

		// keep <br> tags inside captions and remove line breaks
		if ( html.indexOf( '[caption' ) !== -1 ) {
			preserve_br = true;
			html = html.replace( /\[caption[\s\S]+?\[\/caption\]/g, function( a ) {
				return a.replace( /<br([^>]*)>/g, '<wp-temp-br$1>' ).replace( /[\r\n\t]+/, '' );
			});
		}

		// Pretty it up for the source editor
		html = html.replace( new RegExp( '\\s*</(' + blocklist1 + ')>\\s*', 'g' ), '</$1>\n' );
		html = html.replace( new RegExp( '\\s*<((?:' + blocklist1 + ')(?: [^>]*)?)>', 'g' ), '\n<$1>' );

		// Mark </p> if it has any attributes.
		html = html.replace( /(<p [^>]+>.*?)<\/p>/g, '$1</p#>' );

		// Separate <div> containing <p>
		html = html.replace( /<div( [^>]*)?>\s*<p>/gi, '<div$1>\n\n' );

		// Remove <p> and <br />
		html = html.replace( /\s*<p>/gi, '' );
		html = html.replace( /\s*<\/p>\s*/gi, '\n\n' );
		html = html.replace( /\n[\s\u00a0]+\n/g, '\n\n' );
		html = html.replace( /\s*<br ?\/?>\s*/gi, '\n' );

		// Fix some block element newline issues
		html = html.replace( /\s*<div/g, '\n<div' );
		html = html.replace( /<\/div>\s*/g, '</div>\n' );
		html = html.replace( /\s*\[caption([^\[]+)\[\/caption\]\s*/gi, '\n\n[caption$1[/caption]\n\n' );
		html = html.replace( /caption\]\n\n+\[caption/g, 'caption]\n\n[caption' );

		html = html.replace( new RegExp('\\s*<((?:' + blocklist2 + ')(?: [^>]*)?)\\s*>', 'g' ), '\n<$1>' );
		html = html.replace( new RegExp('\\s*</(' + blocklist2 + ')>\\s*', 'g' ), '</$1>\n' );
		html = html.replace( /<((li|dt|dd)[^>]*)>/g, ' \t<$1>' );

		if ( html.indexOf( '<option' ) !== -1 ) {
			html = html.replace( /\s*<option/g, '\n<option' );
			html = html.replace( /\s*<\/select>/g, '\n</select>' );
		}

		if ( html.indexOf( '<hr' ) !== -1 ) {
			html = html.replace( /\s*<hr( [^>]*)?>\s*/g, '\n\n<hr$1>\n\n' );
		}

		if ( html.indexOf( '<object' ) !== -1 ) {
			html = html.replace( /<object[\s\S]+?<\/object>/g, function( a ) {
				return a.replace( /[\r\n]+/g, '' );
			});
		}

		// Unmark special paragraph closing tags
		html = html.replace( /<\/p#>/g, '</p>\n' );
		html = html.replace( /\s*(<p [^>]+>[\s\S]*?<\/p>)/g, '\n$1' );

		// Trim whitespace
		html = html.replace( /^\s+/, '' );
		html = html.replace( /[\s\u00a0]+$/, '' );

		// put back the line breaks in pre|script
		if ( preserve_linebreaks ) {
			html = html.replace( /<wp-line-break>/g, '\n' );
		}

		// and the <br> tags in captions
		if ( preserve_br ) {
			html = html.replace( /<wp-temp-br([^>]*)>/g, '<br$1>' );
		}

		return html;
	}

	// Similar to `wpautop()` in formatting.php
	window.gdlr_core_autop = function( text ) {
		var preserve_linebreaks = false,
			preserve_br = false,
			blocklist = 'table|thead|tfoot|caption|col|colgroup|tbody|tr|td|th|div|dl|dd|dt|ul|ol|li|pre' +
				'|form|map|area|blockquote|address|math|style|p|h[1-6]|hr|fieldset|legend|section' +
				'|article|aside|hgroup|header|footer|nav|figure|figcaption|details|menu|summary';

		// Normalize line breaks
		text = text.replace( /\r\n|\r/g, '\n' );

		if ( text.indexOf( '\n' ) === -1 ) {
			return text;
		}

		if ( text.indexOf( '<object' ) !== -1 ) {
			text = text.replace( /<object[\s\S]+?<\/object>/g, function( a ) {
				return a.replace( /\n+/g, '' );
			});
		}

		text = text.replace( /<[^<>]+>/g, function( a ) {
			return a.replace( /[\n\t ]+/g, ' ' );
		});

		// Protect pre|script tags
		if ( text.indexOf( '<pre' ) !== -1 || text.indexOf( '<script' ) !== -1 ) {
			preserve_linebreaks = true;
			text = text.replace( /<(pre|script)[^>]*>[\s\S]*?<\/\1>/g, function( a ) {
				return a.replace( /\n/g, '<wp-line-break>' );
			});
		}

		// keep <br> tags inside captions and convert line breaks
		if ( text.indexOf( '[caption' ) !== -1 ) {
			preserve_br = true;
			text = text.replace( /\[caption[\s\S]+?\[\/caption\]/g, function( a ) {
				// keep existing <br>
				a = a.replace( /<br([^>]*)>/g, '<wp-temp-br$1>' );
				// no line breaks inside HTML tags
				a = a.replace( /<[^<>]+>/g, function( b ) {
					return b.replace( /[\n\t ]+/, ' ' );
				});
				// convert remaining line breaks to <br>
				return a.replace( /\s*\n\s*/g, '<wp-temp-br />' );
			});
		}

		text = text + '\n\n';
		text = text.replace( /<br \/>\s*<br \/>/gi, '\n\n' );
		text = text.replace( new RegExp( '(<(?:' + blocklist + ')(?: [^>]*)?>)', 'gi' ), '\n$1' );
		text = text.replace( new RegExp( '(</(?:' + blocklist + ')>)', 'gi' ), '$1\n\n' );
		text = text.replace( /<hr( [^>]*)?>/gi, '<hr$1>\n\n' ); // hr is self closing block element
		text = text.replace( /\s*<option/gi, '<option' ); // No <p> or <br> around <option>
		text = text.replace( /<\/option>\s*/gi, '</option>' );
		text = text.replace( /\n\s*\n+/g, '\n\n' );
		text = text.replace( /([\s\S]+?)\n\n/g, '<p>$1</p>\n' );
		text = text.replace( /<p>\s*?<\/p>/gi, '');
		text = text.replace( new RegExp( '<p>\\s*(</?(?:' + blocklist + ')(?: [^>]*)?>)\\s*</p>', 'gi' ), '$1' );
		text = text.replace( /<p>(<li.+?)<\/p>/gi, '$1');
		text = text.replace( /<p>\s*<blockquote([^>]*)>/gi, '<blockquote$1><p>');
		text = text.replace( /<\/blockquote>\s*<\/p>/gi, '</p></blockquote>');
		text = text.replace( new RegExp( '<p>\\s*(</?(?:' + blocklist + ')(?: [^>]*)?>)', 'gi' ), '$1' );
		text = text.replace( new RegExp( '(</?(?:' + blocklist + ')(?: [^>]*)?>)\\s*</p>', 'gi' ), '$1' );

		// Remove redundant spaces and line breaks after existing <br /> tags
		text = text.replace( /(<br[^>]*>)\s*\n/gi, '$1' );

		// Create <br /> from the remaining line breaks
		text = text.replace( /\s*\n/g, '<br />\n');

		text = text.replace( new RegExp( '(</?(?:' + blocklist + ')[^>]*>)\\s*<br />', 'gi' ), '$1' );
		text = text.replace( /<br \/>(\s*<\/?(?:p|li|div|dl|dd|dt|th|pre|td|ul|ol)>)/gi, '$1' );
		text = text.replace( /(?:<p>|<br ?\/?>)*\s*\[caption([^\[]+)\[\/caption\]\s*(?:<\/p>|<br ?\/?>)*/gi, '[caption$1[/caption]' );

		text = text.replace( /(<(?:div|th|td|form|fieldset|dd)[^>]*>)(.*?)<\/p>/g, function( a, b, c ) {
			if ( c.match( /<p( [^>]*)?>/ ) ) {
				return a;
			}

			return b + '<p>' + c + '</p>';
		});

		// put back the line breaks in pre|script
		if ( preserve_linebreaks ) {
			text = text.replace( /<wp-line-break>/g, '\n' );
		}

		if ( preserve_br ) {
			text = text.replace( /<wp-temp-br([^>]*)>/g, '<br$1>' );
		}

		return text;
	}

})(jQuery);	