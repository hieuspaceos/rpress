<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('product', 'gdlr_core_pb_element_product'); 
	
	if( !class_exists('gdlr_core_pb_element_product') ){
		class gdlr_core_pb_element_product{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-shopping-cart',
					'title' => esc_html__('Product (Woocommerce)', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return apply_filters('gdlr_core_product_item_options', array(					
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'enable-category-filter' => array(
								'title' => esc_html__('Enable Category Filter', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'enable' => esc_html__('Box Style', 'goodlayers-core'),
									'text' => esc_html__('Text Style', 'goodlayers-core'),
									'dropdown' => esc_html__('Dropdown', 'goodlayers-core'),
									'disable' => esc_html__('Disable', 'goodlayers-core')
								),
								'default' => 'disable'
							),
							'filterer-align' => array(
								'title' => esc_html__('Filterer Alignment', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'left',
								'condition' => array('enable-category-filter' => array('enable', 'text'))
							),
							'filterer-all-icon' => array(
								'title' => esc_html__('Filterer All Icon', 'goodlayers-core'),
								'type' => 'icons',
								'default' => 'none',
								'allow-none' => true,
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'category' => array(
								'title' => esc_html__('Category', 'goodlayers-core'),
								'type' => 'multi-combobox',
								'options' => gdlr_core_get_term_list('product_cat'),
								'description' => esc_html__('You can use Ctrl/Command button to select multiple items or remove the selected item. Leave this field blank to select all items in the list.', 'goodlayers-core'),
							),
							'tag' => array(
								'title' => esc_html__('Tag', 'goodlayers-core'),
								'type' => 'multi-combobox',
								'options' => gdlr_core_get_term_list('product_tag')
							),
							'relation' => array(
								'title' => esc_html__('Relation (Category & Tag)', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'or' => esc_html__('OR', 'goodlayers-core'),
									'and' => esc_html__('AND', 'goodlayers-core')
								)
							),
							'num-fetch' => array(
								'title' => esc_html__('Num Fetch', 'goodlayers-core'),
								'type' => 'text',
								'default' => 9,
								'data-input-type' => 'number',
								'description' => esc_html__('The number of posts showing on the product item', 'goodlayers-core')
							),
							'stock-status' => array(
								'title' => esc_html__('Stock Status', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'all' => esc_html__('All', 'goodlayers-core'), 
									'instock' => esc_html__('In Stock', 'goodlayers-core'), 
									'outofstock' => esc_html__('Out of Stock', 'goodlayers-core'), 
									'onrequest' => esc_html__('On Request', 'goodlayers-core'), 
								)
							),
							'orderby' => array(
								'title' => esc_html__('Order By', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'date' => esc_html__('Publish Date', 'goodlayers-core'), 
									'title' => esc_html__('Title', 'goodlayers-core'), 
									'rand' => esc_html__('Random', 'goodlayers-core'), 
									'menu_order' => esc_html__('Menu Order', 'goodlayers-core'), 
								)
							),
							'order' => array(
								'title' => esc_html__('Order', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'desc'=>esc_html__('Descending Order', 'goodlayers-core'), 
									'asc'=> esc_html__('Ascending Order', 'goodlayers-core'), 
								)
							),
							'pagination' => array(
								'title' => esc_html__('Pagination', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none'=>esc_html__('None', 'goodlayers-core'), 
									'page'=>esc_html__('Page', 'goodlayers-core'), 
									'load-more'=>esc_html__('Load More', 'goodlayers-core'), 
								),
								'description' => esc_html__('Pagination is not supported and will be automatically disabled on carousel layout.', 'goodlayers-core'),
							),
							'pagination-style' => array(
								'title' => esc_html__('Pagination Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'default' => esc_html__('Default', 'goodlayers-core'),
									'plain' => esc_html__('Plain', 'goodlayers-core'),
									'rectangle' => esc_html__('Rectangle', 'goodlayers-core'),
									'rectangle-border' => esc_html__('Rectangle Border', 'goodlayers-core'),
									'round' => esc_html__('Round', 'goodlayers-core'),
									'round-border' => esc_html__('Round Border', 'goodlayers-core'),
									'circle' => esc_html__('Circle', 'goodlayers-core'),
									'circle-border' => esc_html__('Circle Border', 'goodlayers-core'),
								),
								'default' => 'default',
								'condition' => array( 'pagination' => 'page' )
							),
							'pagination-align' => array(
								'title' => esc_html__('Pagination Alignment', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'with-default' => true,
								'default' => 'default',
								'condition' => array( 'pagination' => 'page' )
							),

						),
					),
					'settings' => array(
						'title' => esc_html__('Product Style', 'goodlayers-core'),
						'options' => array(
							'product-style' => array(
								'title' => esc_html__('Product Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'grid' => esc_html__('Grid', 'goodlayers-core'),
									'grid-2' => esc_html__('Grid 2', 'goodlayers-core'),
									'grid-3' => esc_html__('Grid 3', 'goodlayers-core'),
									'grid-3-with-border' => esc_html__('Grid 3 With Border', 'goodlayers-core'),
									'grid-3-without-frame' => esc_html__('Grid 3 Without Frame', 'goodlayers-core'),
									'grid-4' => esc_html__('Grid 4', 'goodlayers-core'),
									'grid-5' => esc_html__('Grid 5', 'goodlayers-core'),
									'grid-6' => esc_html__('Grid 6', 'goodlayers-core'),
									'box' => esc_html__('Box', 'goodlayers-core')
								),
							),
							'enable-rating' => array(
								'title' => esc_html__('Enable Rating', 'goodlayers-core'),
								'type' => 'checkbox', 
								'default' => 'enable',
								'condition'  => array('product-style' => 'grid-2')
							),
							'attribute-field' => array(
								'title' => esc_html__('Attribute Field', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array('product-style' => 'box'),	
								'description' => esc_html__('Fill the attribute separated by | sign. Eg. "Type | THC | CBC"', 'goodlayers-core')
							
							),
							'variation-field' => array(
								'title' => esc_html__('Variation Field', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array('product-style' => 'box'),
								'description' => esc_html__('Fill the variation separated by | sign. Eg. "1 Oz | 1/8 Oz | 1/4 Oz"', 'goodlayers-core')
							),
							'excerpt-number' => array(
								'title' => esc_html__('Excerpt Number', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'number',
								'default' => 15,
								'condition' => array('product-style' => 'grid-5')
							),
							'price-location' => array(
								'title' => esc_html__('Price Location', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'after-title' => esc_html__('After Title', 'goodlayers-core'),
									'thumbnail' => esc_html__('Thumbnail', 'goodlayers-core')
								),
								'condition' => array('product-style' => 'grid-2')
							),
							'display-attribute-amount' => array(
								'title' => esc_html__('Display Attribute Amount (Number Only)', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'product-style' => array('grid-3', 'grid-3-with-border', 'grid-3-without-frame') )
							),
							'button-style' => array(
								'title' => esc_html__('Button Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'plain' => esc_html__('Plain Text', 'goodlayers-core'),
									'border' => esc_html__('Border Button', 'goodlayers-core'),
									'thumbnail' => esc_html__('Thumbnail', 'goodlayers-core'),
									'thumbnail-modern' => esc_html__('Thumbnail Modern', 'goodlayers-core')
								),
								'condition' => array( 'product-style' => array('grid-3', 'grid-3-with-border', 'grid-3-without-frame') )
							),
							'button-border-color' => array(
								'title' => esc_html__('Button Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'product-style' => array('grid-3', 'grid-3-with-border', 'grid-3-without-frame'), 'button-style' => array('plain', 'border') )
							),
							'thumbnail-size' => array(
								'title' => esc_html__('Thumbnail Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size',
								'condition' => array('product-style' => array('grid', 'grid-2', 'grid-3', 'grid-3-with-border', 'grid-3-without-frame', 'grid-4', 'grid-5', 'grid-6')),
							),
							'enable-thumbnail-link' => array(
								'title' => esc_html__('Enable Thumbnail Link', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable'
							),
							'enable-thumbnail-zoom-on-hover' => array(
								'title' => esc_html__('Thumbnail Zoom on Hover', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array('product-style' => array('grid', 'grid-2', 'grid-3', 'grid-3-with-border', 'grid-3-without-frame', 'grid-4', 'grid-5', 'grid-6')),
							),
							'enable-thumbnail-grayscale-effect' => array(
								'title' => esc_html__('Enable Thumbnail Grayscale Effect', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array('product-style' => array('grid', 'grid-2', 'grid-3', 'grid-3-with-border', 'grid-3-without-frame', 'grid-4', 'grid-5', 'grid-6')),
								'description' => esc_html__('Only works with browser that supports css3 filter ( http://caniuse.com/#feat=css-filters ).', 'goodlayers-core')
							),
							'column-size' => array(
								'title' => esc_html__('Column Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 60 => 1, 30 => 2, 20 => 3, 15 => 4, 12 => 5 ),
								'default' => 20,
							),
							'layout' => array(
								'title' => esc_html__('Layout', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 
									'fitrows' => esc_html__('Fit Rows', 'goodlayers-core'),
									'carousel' => esc_html__('Carousel', 'goodlayers-core'),
									'masonry' => esc_html__('Masonry', 'goodlayers-core'),
								),
								'default' => 'fitrows',
							),
							'carousel-overflow' => array(
								'title' => esc_html__('Carousel Overflow', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Hidden', 'goodlayers-core'),
									'visible' => esc_html__('Visible', 'goodlayers-core')
								),
								'condition' => array( 'layout' => 'carousel' )
							),
							'carousel-autoslide' => array(
								'title' => esc_html__('Autoslide Carousel', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array( 'layout' => 'carousel' )
							),
							'carousel-scrolling-item-amount' => array(
								'title' => esc_html__('Carousel Scrolling Item Amount', 'goodlayers-core'),
								'type' => 'text',
								'default' => '1',
								'condition' => array( 'layout' => 'carousel' )
							),
							'carousel-navigation' => array(
								'title' => esc_html__('Carousel Navigation', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'navigation' => esc_html__('Only Navigation', 'goodlayers-core'),
									'touch-navigation' => esc_html__('Touch Navigation', 'goodlayers-core'),
									'touch-navigation-bullet' => esc_html__('Touch Navigation & Bullet', 'goodlayers-core'),
									'navigation-outer-plain-round' => esc_html__('Only ( Outer ) Navigation Plain Round Style', 'goodlayers-core'),
									'bullet' => esc_html__('Only Bullet', 'goodlayers-core'),
									'both' => esc_html__('Both Navigation and Bullet', 'goodlayers-core'),

									'navigation-top' => esc_html__('Navigation Top (Custom)', 'goodlayers-core'),
									'navigation-bottom' => esc_html__('Navigation Bottom (Custom)', 'goodlayers-core'),
									'navigation-outer' => esc_html__('Navigation Outer (Custom)', 'goodlayers-core'),
									'navigation-inner' => esc_html__('Navigation Inner (Custom)', 'goodlayers-core'),
								),
								'default' => 'navigation',
								'condition' => array( 'layout' => 'carousel' )
							),
							'carousel-navigation-show-on-hover' => array(
								'title' => esc_html__('Carousel Navigation Display On Hover', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'carousel-navigation' => array('navigation-outer', 'navigation-inner') )
							),
							'carousel-navigation-align' => (function_exists('gdlr_core_get_flexslider_navigation_align')? gdlr_core_get_flexslider_navigation_align(): array()),
							'carousel-navigation-left-icon' => (function_exists('gdlr_core_get_flexslider_navigation_left_icon')? gdlr_core_get_flexslider_navigation_left_icon(): array()),
							'carousel-navigation-right-icon' => (function_exists('gdlr_core_get_flexslider_navigation_right_icon')? gdlr_core_get_flexslider_navigation_right_icon(): array()),
							'carousel-navigation-icon-color' => (function_exists('gdlr_core_get_flexslider_navigation_icon_color')? gdlr_core_get_flexslider_navigation_icon_color(): array()),
							'carousel-navigation-icon-hover-color' => (function_exists('gdlr_core_get_flexslider_navigation_icon_hover_color')? gdlr_core_get_flexslider_navigation_icon_hover_color(): array()),
							'carousel-navigation-icon-bg' => (function_exists('gdlr_core_get_flexslider_navigation_icon_background')? gdlr_core_get_flexslider_navigation_icon_background(): array()),
							'carousel-navigation-icon-padding' => (function_exists('gdlr_core_get_flexslider_navigation_icon_padding')? gdlr_core_get_flexslider_navigation_icon_padding(): array()),
							'carousel-navigation-icon-radius' => (function_exists('gdlr_core_get_flexslider_navigation_icon_radius')? gdlr_core_get_flexslider_navigation_icon_radius(): array()),
							'carousel-navigation-size' => (function_exists('gdlr_core_get_flexslider_navigation_icon_size')? gdlr_core_get_flexslider_navigation_icon_size(): array()),
							'carousel-navigation-margin' => (function_exists('gdlr_core_get_flexslider_navigation_margin')? gdlr_core_get_flexslider_navigation_margin(): array()),
							'carousel-navigation-icon-margin' => (function_exists('gdlr_core_get_flexslider_navigation_icon_margin')? gdlr_core_get_flexslider_navigation_icon_margin(): array()),
							'carousel-bullet-style' => array(
								'title' => esc_html__('Carousel Bullet Style', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => (function_exists('gdlr_core_get_flexslider_bullet_itypes')? gdlr_core_get_flexslider_bullet_itypes(): array()),
								'condition' => array( 'layout' => 'carousel', 'carousel-navigation' => array('bullet','both','touch-navigation-bullet') ),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'carousel-bullet-top-margin' => array(
								'title' => esc_html__('Carousel Bullet Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'layout' => 'carousel', 'carousel-navigation' => array('bullet','both','touch-navigation-bullet') )
							),
						),
					),	
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'product-title-font-size' => array(
								'title' => esc_html__('Product Title Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'product-title-font-weight' => array(
								'title' => esc_html__('Product Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'product-title-letter-spacing' => array(
								'title' => esc_html__('Product Title Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),							
							'product-title-text-transform' => array(
								'title' => esc_html__('Product Title Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'data-type' => 'text',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								),
								'default' => 'none'
							),
						),
					),
					'shadow' => array(
						'title' => esc_html__('Border/Shadow', 'goodlayers-core'),
						'options' => array(
							'border-radius' => array(
								'title' => esc_html__('Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'description' => esc_html__('Only for grid 5 style', 'goodlayers-core')
							),
							'frame-padding' => array(
								'title' => esc_html__('Frame Padding', 'tourmaster'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'unlink' ),
							),
							'frame-border-size' => array(
								'title' => esc_html__('Frame Border Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'frame-border-color' => array(
								'title' => esc_html__('Frame Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'frame-shadow-size' => array(
								'title' => esc_html__('Shadow Size ( for image/frame )', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
							),
							'frame-shadow-color' => array(
								'title' => esc_html__('Shadow Color ( for image/frame )', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'frame-shadow-opacity' => array(
								'title' => esc_html__('Shadow Opacity ( for image/frame )', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
							'enable-move-up-shadow-effect' => array(
								'title' => esc_html__('Move Up Shadow Hover Effect', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'description' => esc_html__('Only effects the "Grid 3" style', 'goodlayers-core')
							),
							'move-up-effect-length' => array(
								'title' => esc_html__('Move Up Hover Effect Length', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
							'frame-hover-shadow-size' => array(
								'title' => esc_html__('Shadow Hover Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
							'frame-hover-shadow-color' => array(
								'title' => esc_html__('Shadow Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
							'frame-hover-shadow-opacity' => array(
								'title' => esc_html__('Shadow Hover Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
								'condition' => array( 'enable-move-up-shadow-effect' => 'enable' )
							),
						),
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'product-title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'product-title-hover-color' => array(
								'title' => esc_html__('Title Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'product-caption-color' => array(
								'title' => esc_html__('Box Caption Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'product-caption-hover-color' => array(
								'title' => esc_html__('Box Caption Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'box-background-color' => array(
								'title' => esc_html__('Box Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'box-background-hover-color' => array(
								'title' => esc_html__('Box Background Hover Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						),
					),
					'item-title' => array(
						'title' => esc_html__('Item Title', 'goodlayers-core'),
						'options' => gdlr_core_block_item_option()
					)
				));
			}

			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-product-<?php echo esc_attr($id); ?>" >
if( document.readyState == 'complete' ){
	jQuery(document).ready(function(){
		jQuery('#gdlr-core-preview-product-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider().gdlr_core_isotope();
	});
}else{
	jQuery(window).on('load', function(){
		setTimeout(function(){
			jQuery('#gdlr-core-preview-product-<?php echo esc_attr($id); ?>').parent().gdlr_core_flexslider().gdlr_core_isotope();
		}, 300);
	});
}
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'category' => '', 'tag' => '', 'num-fetch' => '9', 'thumbnail-size' => 'full', 'orderby' => 'date', 'order' => 'desc',
						'column-size' => 20, 'layout' => 'fitrows',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				$settings['product-style'] = empty($settings['product-style'])? 'grid': $settings['product-style'];
				$settings['no-space'] = 'no';
				$settings['layout'] = empty($settings['layout'])? 'fitrows': $settings['layout'];
				$settings['has-column'] = 'yes';

				$custom_style = '';
				
				if( !empty($settings['carousel-navigation-icon-hover-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-flexslider-custom-nav i:hover{ color: ' . $settings['carousel-navigation-icon-hover-color'] . ' !important; }';
				}

				if( in_array($settings['product-style'], array('grid-3', 'grid-3-with-border')) && !empty($settings['button-border-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-product-grid-3.gdlr-core-button-style-border .gdlr-core-product-add-to-cart{ color: ' . $settings['button-border-color'] . '; border-color: ' . $settings['button-border-color'] . '; }';
					$custom_style .= '#custom_style_id .gdlr-core-product-grid-3.gdlr-core-button-style-border .gdlr-core-product-add-to-cart:hover{ background-color: ' . $settings['button-border-color'] . '; color: #fff; }';
				}

				if( in_array($settings['product-style'], array('grid-3', 'grid-3-with-border', 'grid-5')) ){
					if( !empty($settings['blog-frame-hover-border-color']) ){
						$custom_style .= '#custom_style_id .gdlr-core-blog-grid.gdlr-core-blog-grid-with-frame:hover{ border-color: ' . $settings['blog-frame-hover-border-color'] . ' !important; }';
					}
					if( !empty($settings['enable-move-up-shadow-effect']) && $settings['enable-move-up-shadow-effect'] == 'enable' ){
						$custom_style_temp = gdlr_core_esc_style(array(
							'background-shadow-size' => empty($settings['frame-hover-shadow-size'])? '': $settings['frame-hover-shadow-size'],
							'background-shadow-color' => empty($settings['frame-hover-shadow-color'])? '': $settings['frame-hover-shadow-color'],
							'background-shadow-opacity' => empty($settings['frame-hover-shadow-opacity'])? '': $settings['frame-hover-shadow-opacity'],
						), false, true);

						if( !empty($settings['move-up-effect-length']) ){
							$custom_style_temp .= 'transform: translate3d(0, -' . $settings['move-up-effect-length'] . ', 0); ';
						}

						if( !empty($custom_style_temp) ){
							$custom_style .= '#custom_style_id .gdlr-core-move-up-with-shadow:hover{ ' . $custom_style_temp . ' }';
						}
					}
				}

				if( !empty($settings['product-title-hover-color']) ){
					$custom_style .= '#custom_style_id.gdlr-core-product-item .gdlr-core-product-title a:hover{ color: ' . $settings['product-title-hover-color'] . ' !important; }';
					$custom_style .= '#custom_style_id .gdlr-core-product-box:hover .gdlr-core-product-title a{ color: ' . $settings['product-title-hover-color'] . ' !important; }';
				}
				if( !empty($settings['product-caption-hover-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-product-box:hover .gdlr-core-product-attribute{ color: ' . $settings['product-caption-hover-color'] . ' !important; }';
				}
				if( !empty($settings['box-background-hover-color']) ){
					$custom_style .= '#custom_style_id .gdlr-core-product-box:hover{ background-color: ' . $settings['box-background-hover-color'] . ' !important; }';
				}

				if( !empty($custom_style) ){
					if( empty($settings['id']) ){
						global $gdlr_core_product_id; 

						if( $preview ){
							$gdlr_core_product_id = empty($gdlr_core_product_id)? array(): $gdlr_core_product_id;

							// generate unique id so it does not get overwritten in admin area
							$rnd_product_id = mt_rand(0, 99999);
							while( in_array($rnd_product_id, $gdlr_core_product_id) ){
								$rnd_product_id = mt_rand(0, 99999);
							}
							$gdlr_core_product_id[] = $rnd_product_id;
							$settings['id'] = 'gdlr-core-product-' . $rnd_product_id;
						}else{
							$gdlr_core_product_id = empty($gdlr_core_product_id)? 1: $gdlr_core_product_id + 1;
							$settings['id'] = 'gdlr-core-product-' . $gdlr_core_product_id;
						}
					}

					$custom_style = str_replace('custom_style_id', $settings['id'], $custom_style); 
					if( $preview ){
						$custom_style = '<style>' . $custom_style . '</style>';
					}else{
						gdlr_core_add_inline_style($custom_style);
						$custom_style = '';
					}
				}

				// start printing item
				$extra_class = ' gdlr-core-product-style-' . $settings['product-style'];
				$title_settings = $settings;
				if( $settings['no-space'] == 'yes' || $settings['layout'] == 'carousel' ){
					$title_settings['pdlr'] = false;
					$extra_class .= ' gdlr-core-item-pdlr';
				}
				if( $settings['layout'] == 'carousel' ){
					if( empty($settings['carousel-navigation']) || in_array($settings['carousel-navigation'], array('navigation', 'both')) ){
						$title_settings['carousel'] = 'enable';
					}
				}
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];

				$ret  = '<div class="woocommerce gdlr-core-product-item gdlr-core-item-pdb clearfix ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( class_exists('WooCommerce') ){
					
					// print title
					$ret .= gdlr_core_block_item_title($title_settings);
					
					// pring product item
					$product_item = new gdlr_core_product_item($settings);

					$ret .= $product_item->get_content();

				}else{
					$ret .= '<div class="gdlr-core-external-plugin-message">' . esc_html__('Please install and activate the "Woocommerce" plugin before using this item.', 'goodlayers-core') . '</div>';
				}
				
				$ret .= '</div>'; // gdlr-core-product-item
				$ret .= $custom_style;

				return $ret;
			}			
			
		} // gdlr_core_pb_element_product
	} // class_exists	